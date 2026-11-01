# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.8](https://github.com/open-lakehouse/mangrove/compare/olai-uc-sqlite-v0.0.7...olai-uc-sqlite-v0.0.8) - 2026-11-01

### Added

- *(storage-access)* [**breaking**] own the Postgres and SQLite backends (#277)

## [0.0.7](https://github.com/open-lakehouse/mangrove/compare/olai-uc-sqlite-v0.0.6...olai-uc-sqlite-v0.0.7) - 2026-10-31

### Performance

- [**breaking**] standalone storage-access store with indexed location resolution (#254) (#273)

## [0.0.2](https://github.com/open-lakehouse/mangrove/compare/olai-uc-sqlite-v0.0.1...olai-uc-sqlite-v0.0.2) - 2026-07-12

### Added

- adopt olai-store SqlStore/PgStore, inline credential secrets (#46)
- extract the UC Delta v1 API into the olai-uc-delta-api crate (#37)

### Changed

- consolidate storage abstractions on olai-store 0.0.6 (#48)
