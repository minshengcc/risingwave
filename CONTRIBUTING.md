# Contribution guidelines

Thanks for your interest in contributing to RisingWave! We welcome and appreciate contributions.

This document describes how to submit your code changes. To learn about the development process, see the [developer guide](docs/developer-guide.md). To understand the design and implementation of RisingWave, refer to the design docs listed in [docs/README.md](docs/README.md).

If you have questions, you can search for existing discussions or start a new discussion in the [Discussions forum of RisingWave](https://github.com/risingwavelabs/risingwave/discussions), or ask in the RisingWave Community channel on Slack. Please use the [invitation link](https://join.slack.com/t/risingwave-community/shared_invite/zt-120rft0mr-d8uGk3d~NZiZAQWPnElOfw) to join the channel.

To report bugs, create a [GitHub issue](https://github.com/risingwavelabs/risingwave/issues/new/choose).


## Table of contents

- [Contribution guidelines](#contribution-guidelines)
  - [Table of contents](#table-of-contents)
  - [Tests and miscellaneous checks](#tests-and-miscellaneous-checks)
  - [Submit a PR](#submit-a-pr)
    - [Pull Request title](#pull-request-title)
    - [Pull Request description](#pull-request-description)
    - [Sign the CLA](#sign-the-cla)
    - [Cherry pick the commit to release candidate branch](#cherry-pick-the-commit-to-release-candidate-branch)

## Tests and miscellaneous checks

Before submitting your code changes, ensure you fully test them and perform necessary checks. The testing instructions and necessary checks are detailed in the [developer guide](docs/developer-guide.md#test-your-code-changes).

## Submit a PR

### Pull Request title

As described in [here](https://github.com/commitizen/conventional-commit-types/blob/master/index.json), a valid PR title should begin with one of the following prefixes:

- `feat`: A new feature
- `fix`: A bug fix
- `doc`: Documentation only changes
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `style`: A refactoring that improves code style
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies (example scopes: `.config`, `.cargo`, `Cargo.toml`)
- `ci`: Changes to RisingWave CI configuration files and scripts (example scopes: `.github`, `ci` (Buildkite))
- `chore`: Other changes that don't modify src or test files
- `revert`: Reverts a previous commit

For example, a PR title could be:

- `refactor: modify executor protobuf package path`
- `feat(execution): enable comparison between nullable data arrays`, where `(execution)` means that this PR mainly focuses on the execution component.

You may also check out previous PRs in the [PR list](https://github.com/risingwavelabs/risingwave/pulls).

### Pull Request description

- If your PR is small (such as a typo fix), you can go brief.
- If it is large and you have changed a lot, it's better to write more details.

### Sign the CLA

Contributors will need to sign RisingWave Labs' CLA.

### Cherry pick the commit to release candidate branch
We have a GitHub Action to help cherry-pick commits from `main` branch to a `release candidate` branch, such as `v*.*.*-rc` where `*` is a number.

Checkout details at: https://github.com/risingwavelabs/risingwave/blob/main/.github/workflows/cherry-pick-to-release-branch.yml

To trigger the action, we give a correct label to the PR on `main` branch :
https://github.com/risingwavelabs/risingwave/blob/main/.github/workflows/cherry-pick-to-release-branch.yml#L10

It will act when the PR on `main` branch merged:
- If `git cherry-pick` does not find any conflicts, it will open a PR to the `release candidate` branch, and assign the original author as the reviewer.

- If there is a conflict, it will open an issue and make the original author the assignee.