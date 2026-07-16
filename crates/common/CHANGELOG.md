# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/open-lakehouse/mangrove/compare/olai-uc-common-v0.0.2...olai-uc-common-v0.0.3) - 2026-07-16

### Added

- add registered models + model versions API (#148) (#76)
- [**breaking**] migrate proto runtime from prost to buffa (supersedes #1) (#53)

### Changed

- *(acceptance)* [**breaking**] replace journey framework with an API-coverage conformance battery (#64)

### Fixed

- *(models)* [**breaking**] flatten registered-model & model-version create bodies (#78)
- *(functions)* [**breaking**] wrap CreateFunction body in a function_info envelope (#75)

## [0.0.2](https://github.com/open-lakehouse/mangrove/compare/olai-uc-common-v0.0.1...olai-uc-common-v0.0.2) - 2026-07-12

### Added

- adopt olai-store SqlStore/PgStore, inline credential secrets (#46)
- extract the UC Delta v1 API into the olai-uc-delta-api crate (#37)
- ABAC /policies endpoint (Databricks Policies API) (#22)

### Changed

- consolidate storage abstractions on olai-store 0.0.6 (#48)

### Documentation

- crate READMEs and rustdoc for delta-api, common, client, object-store (#45)
