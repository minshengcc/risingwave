CREATE TABLE supplier (s_suppkey INT, s_name CHARACTER VARYING, s_address CHARACTER VARYING, s_nationkey INT, s_phone CHARACTER VARYING, s_acctbal NUMERIC, s_comment CHARACTER VARYING, PRIMARY KEY (s_suppkey));
CREATE TABLE part (p_partkey INT, p_name CHARACTER VARYING, p_mfgr CHARACTER VARYING, p_brand CHARACTER VARYING, p_type CHARACTER VARYING, p_size INT, p_container CHARACTER VARYING, p_retailprice NUMERIC, p_comment CHARACTER VARYING, PRIMARY KEY (p_partkey));
CREATE TABLE partsupp (ps_partkey INT, ps_suppkey INT, ps_availqty INT, ps_supplycost NUMERIC, ps_comment CHARACTER VARYING, PRIMARY KEY (ps_partkey, ps_suppkey));
CREATE TABLE customer (c_custkey INT, c_name CHARACTER VARYING, c_address CHARACTER VARYING, c_nationkey INT, c_phone CHARACTER VARYING, c_acctbal NUMERIC, c_mktsegment CHARACTER VARYING, c_comment CHARACTER VARYING, PRIMARY KEY (c_custkey));
CREATE TABLE orders (o_orderkey BIGINT, o_custkey INT, o_orderstatus CHARACTER VARYING, o_totalprice NUMERIC, o_orderdate DATE, o_orderpriority CHARACTER VARYING, o_clerk CHARACTER VARYING, o_shippriority INT, o_comment CHARACTER VARYING, PRIMARY KEY (o_orderkey));
CREATE TABLE lineitem (l_orderkey BIGINT, l_partkey INT, l_suppkey INT, l_linenumber INT, l_quantity NUMERIC, l_extendedprice NUMERIC, l_discount NUMERIC, l_tax NUMERIC, l_returnflag CHARACTER VARYING, l_linestatus CHARACTER VARYING, l_shipdate DATE, l_commitdate DATE, l_receiptdate DATE, l_shipinstruct CHARACTER VARYING, l_shipmode CHARACTER VARYING, l_comment CHARACTER VARYING, PRIMARY KEY (l_orderkey, l_linenumber));
CREATE TABLE nation (n_nationkey INT, n_name CHARACTER VARYING, n_regionkey INT, n_comment CHARACTER VARYING, PRIMARY KEY (n_nationkey));
CREATE TABLE region (r_regionkey INT, r_name CHARACTER VARYING, r_comment CHARACTER VARYING, PRIMARY KEY (r_regionkey));
CREATE TABLE person (id BIGINT, name CHARACTER VARYING, email_address CHARACTER VARYING, credit_card CHARACTER VARYING, city CHARACTER VARYING, state CHARACTER VARYING, date_time TIMESTAMP, extra CHARACTER VARYING, PRIMARY KEY (id));
CREATE TABLE auction (id BIGINT, item_name CHARACTER VARYING, description CHARACTER VARYING, initial_bid BIGINT, reserve BIGINT, date_time TIMESTAMP, expires TIMESTAMP, seller BIGINT, category BIGINT, extra CHARACTER VARYING, PRIMARY KEY (id));
CREATE TABLE bid (auction BIGINT, bidder BIGINT, price BIGINT, channel CHARACTER VARYING, url CHARACTER VARYING, date_time TIMESTAMP, extra CHARACTER VARYING);
CREATE TABLE alltypes1 (c1 BOOLEAN, c2 SMALLINT, c3 INT, c4 BIGINT, c5 REAL, c6 DOUBLE, c7 NUMERIC, c8 DATE, c9 CHARACTER VARYING, c10 TIME, c11 TIMESTAMP, c13 INTERVAL, c14 STRUCT<a INT>, c15 INT[], c16 CHARACTER VARYING[]);
CREATE TABLE alltypes2 (c1 BOOLEAN, c2 SMALLINT, c3 INT, c4 BIGINT, c5 REAL, c6 DOUBLE, c7 NUMERIC, c8 DATE, c9 CHARACTER VARYING, c10 TIME, c11 TIMESTAMP, c13 INTERVAL, c14 STRUCT<a INT>, c15 INT[], c16 CHARACTER VARYING[]);
CREATE MATERIALIZED VIEW m0 AS SELECT sq_2.col_0 AS col_0, sq_2.col_0 AS col_1, sq_2.col_0 AS col_2 FROM (SELECT t_1.ps_availqty AS col_0, (lower(t_1.ps_comment)) AS col_1 FROM person AS t_0 LEFT JOIN partsupp AS t_1 ON t_0.city = t_1.ps_comment GROUP BY t_1.ps_comment, t_1.ps_availqty) AS sq_2 GROUP BY sq_2.col_0 HAVING false;
CREATE MATERIALIZED VIEW m1 AS SELECT false AS col_0 FROM bid AS t_0 JOIN lineitem AS t_1 ON t_0.auction = t_1.l_orderkey AND (t_1.l_shipmode) IN ((OVERLAY(('MgzEVIFnu2') PLACING t_1.l_returnflag FROM t_1.l_partkey FOR t_1.l_linenumber)), t_1.l_shipmode, (OVERLAY('Sj2ULhgb4T' PLACING 'ax5WkjwMn7' FROM (INT '-2147483648') FOR t_1.l_suppkey)), (CASE WHEN ((SMALLINT '211') < t_1.l_suppkey) THEN t_1.l_linestatus ELSE 'lR7QOkoyy0' END), (TRIM(BOTH (lower((TRIM(t_1.l_shipinstruct)))) FROM 'LNvcvqlLHQ')), 'cygr1UtdZ9', t_1.l_comment, 'jl2s3PZUq4', t_1.l_linestatus, t_1.l_returnflag) GROUP BY t_0.channel, t_1.l_partkey, t_1.l_comment, t_1.l_suppkey, t_0.auction, t_0.extra, t_1.l_tax, t_1.l_discount, t_0.bidder, t_1.l_returnflag HAVING false;
CREATE MATERIALIZED VIEW m2 AS SELECT t_1.r_name AS col_0 FROM part AS t_0 LEFT JOIN region AS t_1 ON t_0.p_type = t_1.r_comment GROUP BY t_0.p_comment, t_1.r_name;
CREATE MATERIALIZED VIEW m3 AS SELECT (((SMALLINT '470') * (761)) / (2147483647)) AS col_0, DATE '2022-04-07' AS col_1 FROM tumble(person, person.date_time, INTERVAL '48') AS tumble_0 WHERE true GROUP BY tumble_0.city, tumble_0.id, tumble_0.state, tumble_0.name HAVING false;
CREATE MATERIALIZED VIEW m4 AS WITH with_0 AS (SELECT t_2.c1 AS col_0, false AS col_1 FROM m0 AS t_1 LEFT JOIN alltypes2 AS t_2 ON t_1.col_0 = t_2.c3 AND t_2.c1 GROUP BY t_2.c1, t_2.c9) SELECT ARRAY[(BIGINT '-9223372036854775808'), (BIGINT '124')] AS col_0 FROM with_0 WHERE true;
CREATE MATERIALIZED VIEW m5 AS SELECT hop_0.seller AS col_0 FROM hop(auction, auction.date_time, INTERVAL '60', INTERVAL '3780') AS hop_0 GROUP BY hop_0.date_time, hop_0.extra, hop_0.seller HAVING true;
CREATE MATERIALIZED VIEW m6 AS SELECT (BIGINT '0') AS col_0 FROM m5 AS t_2 GROUP BY t_2.col_0;
CREATE MATERIALIZED VIEW m7 AS SELECT ((INT '143')) AS col_0, TIME '10:57:59' AS col_1, t_0.n_regionkey AS col_2, (REAL '271') AS col_3 FROM nation AS t_0 JOIN alltypes2 AS t_1 ON t_0.n_comment = t_1.c9 AND ((REAL '474') > (SMALLINT '832')) GROUP BY t_0.n_name, t_0.n_comment, t_1.c16, t_1.c3, t_0.n_nationkey, t_0.n_regionkey;
CREATE MATERIALIZED VIEW m8 AS SELECT true AS col_0, t_1.c11 AS col_1, (INT '256415035') AS col_2, t_0.n_regionkey AS col_3 FROM nation AS t_0 RIGHT JOIN alltypes2 AS t_1 ON t_0.n_name = t_1.c9 AND t_1.c1 GROUP BY t_1.c11, t_0.n_regionkey, t_1.c1;
CREATE MATERIALIZED VIEW m9 AS WITH with_0 AS (SELECT (upper('PAvsrHbBYZ')) AS col_0 FROM supplier AS t_1 RIGHT JOIN person AS t_2 ON t_1.s_name = t_2.email_address GROUP BY t_2.city, t_2.name HAVING false) SELECT DATE '2022-04-11' AS col_0, CAST(NULL AS STRUCT<a BOOLEAN, b TIME>) AS col_1, ((SMALLINT '904') << (INT '1')) AS col_2 FROM with_0;