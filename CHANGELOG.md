# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [0.2.0](https://github.com/mangata-finance/mangata-node/compare/v0.1.0...v0.2.0) (2021-09-15)
- extend block header with extra field seed
- store shuffling seed inside header instead of storage
- remove shuffling related inherent

### [0.1.1](https://github.com/mangata-finance/mangata-node/compare/v0.1.0...v0.1.1) (2021-09-09)
- Fixing mutlinode setup
- Passing already shuffled extrinsics instead of doing so at runtime (due to no easy way to inject shuffling seed without Header modifications)

## 0.1.0 (2021-08-17)

- Added SemVer scripts

### Bug Fixes

- README update on docker usage ([#43](https://github.com/mangata-finance/mangata-node/issues/43)) ([3323afa](https://github.com/mangata-finance/mangata-node/commit/3323afae5a44859997788a4a83f0b3532be2f115))
- simplify string->bytes converision ([3d0f22b](https://github.com/mangata-finance/mangata-node/commit/3d0f22b2495a040698380f6fc3ee4e94ec8515a2))
