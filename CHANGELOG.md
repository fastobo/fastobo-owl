# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/fastobo/fastobo-owl/compare/v0.1.2...HEAD


## [v0.1.2] - 2022-01-22

[v0.1.2]: https://github.com/fastobo/fastobo-owl/compare/v0.1.1...v0.1.2

### Added
- `Syntax` variant to the `Error` enum returned when an invalid URL was created in `IntoOwl`.

### Fixed
- Uncaught panic when converting an `OboDoc` missing an `ontology` header clause.

### Changed
- Detect duplicate `ontology` header clause when converting an `OboDoc`.


## [v0.1.1] - 2022-01-20

[v0.1.1]: https://github.com/fastobo/fastobo-owl/compare/v0.1.0...v0.1.1

### Changed
- Bump required `horned-functional` version to `v0.3.2`.


## [v0.1.0] - 2022-01-20

[v0.1.0]: https://github.com/fastobo/fastobo-owl/compare/836b59e...v0.1.0

Initial release.
