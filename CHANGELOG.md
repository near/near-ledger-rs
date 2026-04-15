# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.3](https://github.com/near/near-ledger-rs/compare/v0.9.2...v0.9.3) - 2026-04-15

### Other

- *(deps)* replace `slipped10` with `near-slip10` ([#32](https://github.com/near/near-ledger-rs/pull/32))

## [0.9.2](https://github.com/near/near-ledger-rs/compare/v0.9.1...v0.9.2) - 2026-03-11

### Added

- Added optional BLE suport (under `ble` feature flag) ([#30](https://github.com/near/near-ledger-rs/pull/30))

### Other

- Update GitHub Actions token for release-plz workflow

## [0.9.1](https://github.com/khorolets/near-ledger-rs/compare/v0.9.0...v0.9.1) - 2025-05-31

### Added

- print apdus for speculos ([#27](https://github.com/khorolets/near-ledger-rs/pull/27))

## [0.9.0](https://github.com/khorolets/near-ledger-rs/compare/v0.8.1...v0.9.0) - 2025-02-07

### Other

- [**breaking**] fixed typo (#25)

## [0.8.1](https://github.com/khorolets/near-ledger-rs/compare/v0.8.0...v0.8.1) - 2024-08-23

### Added
- Added a function to open near app on the connected Ledger device ([#23](https://github.com/khorolets/near-ledger-rs/pull/23))

## [0.8.0](https://github.com/khorolets/near-ledger-rs/compare/v0.7.2...v0.8.0) - 2024-08-21

### Added
- [**breaking**] remove direct near dependencies from near-ledger ([#22](https://github.com/khorolets/near-ledger-rs/pull/22))

### Other
- [**breaking**] updated near-* to 0.25.0 ([#20](https://github.com/khorolets/near-ledger-rs/pull/20))

## [0.7.2](https://github.com/khorolets/near-ledger-rs/compare/v0.7.1...v0.7.2) - 2024-08-08

### Other
- updated near-* crates to allow 0.24.0 in addition to all the previously supported versions ([#18](https://github.com/khorolets/near-ledger-rs/pull/18))

## [0.7.1](https://github.com/khorolets/near-ledger-rs/compare/v0.7.0...v0.7.1) - 2024-06-24

### Other
- Slimmed down the dependencies by disabling default features on `ed25519-dalek` ([#16](https://github.com/khorolets/near-ledger-rs/pull/16))

## [0.7.0](https://github.com/khorolets/near-ledger-rs/compare/v0.6.1...v0.7.0) - 2024-06-21

### Other
- [**breaking**] replace `ed25519-dalek 1.0.1` -> `ed25519-dalek 2.1.1`  ([#14](https://github.com/khorolets/near-ledger-rs/pull/14))

## [0.6.1](https://github.com/khorolets/near-ledger-rs/compare/v0.6.0...v0.6.1) - 2024-06-19

### Other
- Updated near-* to 0.23 ([#12](https://github.com/khorolets/near-ledger-rs/pull/12))

## [0.6.0](https://github.com/khorolets/near-ledger-rs/compare/v0.5.0...v0.6.0) - 2024-06-10

### Other
- added release-plz workflow to main and added fmt and clippy check ([#10](https://github.com/khorolets/near-ledger-rs/pull/10))
- [**breaking**] updated deps to the newer version ([#9](https://github.com/khorolets/near-ledger-rs/pull/9))
