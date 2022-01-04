use std::sync::Arc;

use risingwave_common::array::{Row, RowDeserializer};
use risingwave_common::catalog::Schema;
use risingwave_common::error::{ErrorCode, Result};
use risingwave_common::types::{deserialize_datum_from, Datum};
use risingwave_common::util::sort_util::OrderType;
use risingwave_storage::table::{ScannableTable, TableIter};
use risingwave_storage::{Keyspace, StateStore, StateStoreIter};

use super::*;

/// `MViewTable` provides a readable cell-based row table interface,
/// so that data can be queried by AP engine.
pub struct MViewTable<S: StateStore> {
    keyspace: Keyspace<S>,
    schema: Schema,
    pk_columns: Vec<usize>,
    sort_key_serializer: OrderedRowsSerializer,
}

impl<S: StateStore> std::fmt::Debug for MViewTable<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MViewTable")
            .field("schema", &self.schema)
            .field("pk_columns", &self.pk_columns)
            .finish()
    }
}

impl<S: StateStore> MViewTable<S> {
    pub fn new(
        keyspace: Keyspace<S>,
        schema: Schema,
        pk_columns: Vec<usize>,
        orderings: Vec<OrderType>,
    ) -> Self {
        let order_pairs = orderings
            .into_iter()
            .zip(pk_columns.clone().into_iter())
            .collect::<Vec<_>>();
        Self {
            keyspace,
            schema,
            pk_columns,
            sort_key_serializer: OrderedRowsSerializer::new(order_pairs),
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    // TODO(MrCroxx): remove me after iter is impled.
    pub fn storage(&self) -> S {
        self.keyspace.state_store()
    }

    // TODO(MrCroxx): Refactor this after statestore iter is finished.
    pub async fn iter(&self) -> Result<MViewTableIter<S>> {
        Ok(MViewTableIter::new(
            self.keyspace.iter().await?,
            self.keyspace.key().to_owned(),
            self.schema.clone(),
            self.pk_columns.clone(),
        ))
    }

    // TODO(MrCroxx): More interfaces are needed besides cell get.
    pub async fn get(&self, pk: Row, cell_idx: usize) -> Result<Option<Datum>> {
        debug_assert!(cell_idx < self.schema.len());
        // TODO(MrCroxx): More efficient encoding is needed.

        let buf = self
            .keyspace
            .get(
                &[
                    &serialize_pk(&pk, &self.sort_key_serializer)?[..],
                    &serialize_cell_idx(cell_idx as u32)?[..],
                ]
                .concat(),
            )
            .await
            .map_err(|err| ErrorCode::InternalError(err.to_string()))?;

        match buf {
            Some(buf) => {
                let mut deserializer = memcomparable::Deserializer::new(buf);
                let datum = deserialize_datum_from(
                    &self.schema.fields[cell_idx].data_type.data_type_kind(),
                    &mut deserializer,
                )?;
                Ok(Some(datum))
            }
            None => Ok(None),
        }
    }
}

pub struct MViewTableIter<S: StateStore> {
    inner: S::Iter,
    prefix: Vec<u8>,
    schema: Schema,
    pk_columns: Vec<usize>,
}

impl<S: StateStore> MViewTableIter<S> {
    fn new(inner: S::Iter, prefix: Vec<u8>, schema: Schema, pk_columns: Vec<usize>) -> Self {
        Self {
            inner,
            prefix,
            schema,
            pk_columns,
        }
    }
}

#[async_trait::async_trait]
impl<S: StateStore> TableIter for MViewTableIter<S> {
    async fn next(&mut self) -> Result<Option<Row>> {
        let mut pk_buf = vec![];
        let mut restored = 0;
        let mut row_bytes = vec![];
        loop {
            match self.inner.next().await? {
                Some((key, value)) => {
                    // there is no need to deserialize pk in mview

                    if key.len() < self.prefix.len() + 4 {
                        return Err(ErrorCode::InternalError("corrupted key".to_owned()).into());
                    }

                    let cur_pk_buf = &key[self.prefix.len()..key.len() - 4];
                    if restored == 0 {
                        pk_buf = cur_pk_buf.to_owned();
                    } else if pk_buf != cur_pk_buf {
                        // previous item is incomplete
                        return Err(ErrorCode::InternalError("incomplete item".to_owned()).into());
                    }

                    row_bytes.extend_from_slice(&value);

                    restored += 1;
                    if restored == self.schema.len() {
                        break;
                    }

                    // continue loop
                }
                // no more item
                None if restored == 0 => return Ok(None),
                // current item is incomplete
                None => return Err(ErrorCode::InternalError("incomplete item".to_owned()).into()),
            }
        }
        let schema = self
            .schema
            .data_types_clone()
            .into_iter()
            .map(|data_type| data_type.data_type_kind())
            .collect::<Vec<_>>();
        let row_deserializer = RowDeserializer::new(schema);
        let row = row_deserializer.deserialize(&row_bytes)?;
        Ok(Some(row))
    }
}

#[async_trait::async_trait]
impl<S> ScannableTable for MViewTable<S>
where
    S: StateStore,
{
    async fn iter(&self) -> Result<risingwave_storage::table::TableIterRef> {
        Ok(Box::new(self.iter().await?))
    }

    async fn get_data_by_columns(
        &self,
        _column_ids: &[i32],
    ) -> Result<risingwave_storage::bummock::BummockResult> {
        unimplemented!()
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Sync + Send> {
        self
    }

    fn schema(&self) -> Schema {
        self.schema.clone()
    }
}

#[cfg(test)]
mod tests {
    use risingwave_common::catalog::Field;
    use risingwave_common::types::{DataTypeKind, Int32Type, StringType};
    use risingwave_common::util::sort_util::OrderType;
    use risingwave_storage::memory::MemoryStateStore;
    use risingwave_storage::Keyspace;

    use super::*;

    #[tokio::test]
    async fn test_mview_table() {
        let state_store = MemoryStateStore::default();
        let schema = Schema::new(vec![
            Field::new(Int32Type::create(false)),
            Field::new(Int32Type::create(false)),
            Field::new(Int32Type::create(false)),
        ]);
        let pk_columns = vec![0, 1];
        let orderings = vec![OrderType::Ascending, OrderType::Descending];
        let keyspace = Keyspace::executor_root(state_store, 0x42);
        let mut state = ManagedMViewState::new(
            keyspace.clone(),
            schema.clone(),
            pk_columns.clone(),
            orderings.clone(),
        );
        let table = MViewTable::new(keyspace.clone(), schema, pk_columns.clone(), orderings);
        let epoch: u64 = 0;

        state.put(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into()),
            ]),
        );
        state.put(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        );
        state.delete(Row(vec![Some(2_i32.into()), Some(22_i32.into())]));
        state.flush(epoch).await.unwrap();

        let cell_1_0 = table
            .get(Row(vec![Some(1_i32.into()), Some(11_i32.into())]), 0)
            .await
            .unwrap();
        assert!(cell_1_0.is_some());
        assert_eq!(*cell_1_0.unwrap().unwrap().as_int32(), 1);
        let cell_1_1 = table
            .get(Row(vec![Some(1_i32.into()), Some(11_i32.into())]), 1)
            .await
            .unwrap();
        assert!(cell_1_1.is_some());
        assert_eq!(*cell_1_1.unwrap().unwrap().as_int32(), 11);
        let cell_1_2 = table
            .get(Row(vec![Some(1_i32.into()), Some(11_i32.into())]), 2)
            .await
            .unwrap();
        assert!(cell_1_2.is_some());
        assert_eq!(*cell_1_2.unwrap().unwrap().as_int32(), 111);

        let cell_2_0 = table
            .get(Row(vec![Some(2_i32.into()), Some(22_i32.into())]), 0)
            .await
            .unwrap();
        assert!(cell_2_0.is_none());
        let cell_2_1 = table
            .get(Row(vec![Some(2_i32.into()), Some(22_i32.into())]), 1)
            .await
            .unwrap();
        assert!(cell_2_1.is_none());
        let cell_2_2 = table
            .get(Row(vec![Some(2_i32.into()), Some(22_i32.into())]), 2)
            .await
            .unwrap();
        assert!(cell_2_2.is_none());
    }

    #[tokio::test]
    async fn test_mview_table_for_string() {
        let state_store = MemoryStateStore::default();
        let schema = Schema::new(vec![
            Field::new(StringType::create(true, 0, DataTypeKind::Varchar)),
            Field::new(StringType::create(true, 0, DataTypeKind::Varchar)),
            Field::new(StringType::create(true, 0, DataTypeKind::Varchar)),
        ]);
        let pk_columns = vec![0, 1];
        let orderings = vec![OrderType::Ascending, OrderType::Descending];
        let keyspace = Keyspace::executor_root(state_store, 0x42);

        let mut state = ManagedMViewState::new(
            keyspace.clone(),
            schema.clone(),
            pk_columns.clone(),
            orderings.clone(),
        );
        let table = MViewTable::new(keyspace.clone(), schema, pk_columns.clone(), orderings);
        let epoch: u64 = 0;

        state.put(
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
            ]),
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
                Some("111".to_string().into()),
            ]),
        );
        state.put(
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
            ]),
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
                Some("222".to_string().into()),
            ]),
        );
        state.delete(Row(vec![
            Some("2".to_string().into()),
            Some("22".to_string().into()),
        ]));
        state.flush(epoch).await.unwrap();

        let cell_1_0 = table
            .get(
                Row(vec![
                    Some("1".to_string().into()),
                    Some("11".to_string().into()),
                ]),
                0,
            )
            .await
            .unwrap();
        assert!(cell_1_0.is_some());
        assert_eq!(
            Some(cell_1_0.unwrap().unwrap().as_utf8().to_string()),
            Some("1".to_string())
        );
        let cell_1_1 = table
            .get(
                Row(vec![
                    Some("1".to_string().into()),
                    Some("11".to_string().into()),
                ]),
                1,
            )
            .await
            .unwrap();
        assert!(cell_1_1.is_some());
        assert_eq!(
            Some(cell_1_1.unwrap().unwrap().as_utf8().to_string()),
            Some("11".to_string())
        );
        let cell_1_2 = table
            .get(
                Row(vec![
                    Some("1".to_string().into()),
                    Some("11".to_string().into()),
                ]),
                2,
            )
            .await
            .unwrap();
        assert!(cell_1_2.is_some());
        assert_eq!(
            Some(cell_1_2.unwrap().unwrap().as_utf8().to_string()),
            Some("111".to_string())
        );

        let cell_2_0 = table
            .get(
                Row(vec![
                    Some("2".to_string().into()),
                    Some("22".to_string().into()),
                ]),
                0,
            )
            .await
            .unwrap();
        assert!(cell_2_0.is_none());
        let cell_2_1 = table
            .get(
                Row(vec![
                    Some("2".to_string().into()),
                    Some("22".to_string().into()),
                ]),
                1,
            )
            .await
            .unwrap();
        assert!(cell_2_1.is_none());
        let cell_2_2 = table
            .get(
                Row(vec![
                    Some("2".to_string().into()),
                    Some("22".to_string().into()),
                ]),
                2,
            )
            .await
            .unwrap();
        assert!(cell_2_2.is_none());
    }

    #[tokio::test]
    async fn test_mview_table_iter() {
        let state_store = MemoryStateStore::default();
        let schema = Schema::new(vec![
            Field::new(Int32Type::create(false)),
            Field::new(Int32Type::create(false)),
            Field::new(Int32Type::create(false)),
        ]);
        let pk_columns = vec![0, 1];
        let orderings = vec![OrderType::Ascending, OrderType::Descending];
        let keyspace = Keyspace::executor_root(state_store, 0x42);

        let mut state = ManagedMViewState::new(
            keyspace.clone(),
            schema.clone(),
            pk_columns.clone(),
            orderings.clone(),
        );
        let table = MViewTable::new(keyspace.clone(), schema, pk_columns.clone(), orderings);
        let epoch: u64 = 0;

        state.put(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into()),
            ]),
        );
        state.put(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        );
        state.delete(Row(vec![Some(2_i32.into()), Some(22_i32.into())]));
        state.flush(epoch).await.unwrap();

        let mut iter = table.iter().await.unwrap();

        let res = iter.next().await.unwrap();
        assert!(res.is_some());
        assert_eq!(
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into())
            ]),
            res.unwrap()
        );

        let res = iter.next().await.unwrap();
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_multi_mview_table_iter() {
        let state_store = MemoryStateStore::default();
        let schema_1 = Schema::new(vec![
            Field::new(Int32Type::create(false)),
            Field::new(Int32Type::create(false)),
            Field::new(Int32Type::create(false)),
        ]);
        let schema_2 = Schema::new(vec![
            Field::new(StringType::create(true, 0, DataTypeKind::Varchar)),
            Field::new(StringType::create(true, 0, DataTypeKind::Varchar)),
            Field::new(StringType::create(true, 0, DataTypeKind::Varchar)),
        ]);
        let pk_columns = vec![0, 1];
        let orderings = vec![OrderType::Ascending, OrderType::Descending];

        let keyspace_1 = Keyspace::executor_root(state_store.clone(), 0x1111);
        let keyspace_2 = Keyspace::executor_root(state_store.clone(), 0x2222);
        let epoch: u64 = 0;

        let mut state_1 = ManagedMViewState::new(
            keyspace_1.clone(),
            schema_1.clone(),
            pk_columns.clone(),
            orderings.clone(),
        );
        let mut state_2 = ManagedMViewState::new(
            keyspace_2.clone(),
            schema_2.clone(),
            pk_columns.clone(),
            orderings.clone(),
        );

        let table_1 = MViewTable::new(
            keyspace_1.clone(),
            schema_1.clone(),
            pk_columns.clone(),
            orderings.clone(),
        );
        let table_2 = MViewTable::new(
            keyspace_2.clone(),
            schema_2.clone(),
            pk_columns.clone(),
            orderings,
        );

        state_1.put(
            Row(vec![Some(1_i32.into()), Some(11_i32.into())]),
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into()),
            ]),
        );
        state_1.put(
            Row(vec![Some(2_i32.into()), Some(22_i32.into())]),
            Row(vec![
                Some(2_i32.into()),
                Some(22_i32.into()),
                Some(222_i32.into()),
            ]),
        );
        state_1.delete(Row(vec![Some(2_i32.into()), Some(22_i32.into())]));

        state_2.put(
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
            ]),
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
                Some("111".to_string().into()),
            ]),
        );
        state_2.put(
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
            ]),
            Row(vec![
                Some("2".to_string().into()),
                Some("22".to_string().into()),
                Some("222".to_string().into()),
            ]),
        );
        state_2.delete(Row(vec![
            Some("2".to_string().into()),
            Some("22".to_string().into()),
        ]));

        state_1.flush(epoch).await.unwrap();
        state_2.flush(epoch).await.unwrap();

        let mut iter_1 = table_1.iter().await.unwrap();
        let mut iter_2 = table_2.iter().await.unwrap();

        let res_1_1 = iter_1.next().await.unwrap();
        assert!(res_1_1.is_some());
        assert_eq!(
            Row(vec![
                Some(1_i32.into()),
                Some(11_i32.into()),
                Some(111_i32.into()),
            ]),
            res_1_1.unwrap()
        );
        let res_1_2 = iter_1.next().await.unwrap();
        assert!(res_1_2.is_none());

        let res_2_1 = iter_2.next().await.unwrap();
        assert!(res_2_1.is_some());
        assert_eq!(
            Row(vec![
                Some("1".to_string().into()),
                Some("11".to_string().into()),
                Some("111".to_string().into())
            ]),
            res_2_1.unwrap()
        );
        let res_2_2 = iter_2.next().await.unwrap();
        assert!(res_2_2.is_none());
    }
}
