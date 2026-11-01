# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.8](https://github.com/open-lakehouse/mangrove/compare/olai-uc-common-v0.0.7...olai-uc-common-v0.0.8) - 2026-11-01

### Added

- search resources and index storage locations (#283)

### Performance

- index stored location keys (#290)

## [0.0.7](https://github.com/open-lakehouse/mangrove/compare/olai-uc-common-v0.0.6...olai-uc-common-v0.0.7) - 2026-10-31

### Added

- [**breaking**] per-resource features and clients as handlers (#261) (#268)
- restore the shares, recipients and providers models (#264)
- [**breaking**] feature-gate generated models per resource (#263)

## [0.0.6](https://github.com/open-lakehouse/mangrove/compare/olai-uc-common-v0.0.5...olai-uc-common-v0.0.6) - 2026-10-30

### Added

- [**breaking**] align Unity Catalog protos with the Databricks API reference (#251)

### Changed

- [**breaking**] move Open Sharing and the shares/recipients/providers APIs to distributary (#257)
- [**breaking**] adopt trestle credential-vending crates (#247) (#255)
- *(sharing)* [**breaking**] decouple Open Sharing from the Unity Catalog crates (#246)

## [0.0.4](https://github.com/open-lakehouse/mangrove/compare/olai-uc-common-v0.0.3...olai-uc-common-v0.0.4) - 2026-07-26

### Added

- storage byte-proxy — wasm client, reverse-proxy auth, CORS docs (#172)
- make olai-uc-client wasm-capable via dual transport (Azure-first) (#150)
- *(workspace)* converge native+wasm graph, move Delta provider to olai-delta-df, pyo3 0.28 (#122)

### Changed

- *(just)* split the root justfile into task modules (#147)

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
