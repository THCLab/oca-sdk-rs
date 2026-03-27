## [2.0.0] - 2026-03-27

### ⚙️ Miscellaneous Tasks

- Bump oca-rs version to 2.0.0
## [2.0.0-rc.8] - 2026-02-25

### ⚙️ Miscellaneous Tasks

- Bump overlay-file crate
- Refresh cargo lock
- Release 2.0.0-rc.8 version
## [2.0.0-rc.7] - 2026-02-21

### ⚙️ Miscellaneous Tasks

- Remove unused import and run fmt
- Release 2.0.0-rc.7 version
## [2.0.0-rc.6] - 2026-02-21

### ⚙️ Miscellaneous Tasks

- Update README with instruction how to start
- Simplify external interface
- Release 2.0.0-rc.6 version
## [2.0.0-rc.5] - 2026-01-29

### ⚙️ Miscellaneous Tasks

- Improve docs, tests and add examples
- Release 2.0.0-rc.5 version
## [2.0.0-rc.4] - 2026-01-23

### ⚙️ Miscellaneous Tasks

- Release 2.0.0-rc.4 version
## [2.0.0-rc.3] - 2026-01-23

### 🚀 Features

- Remove transformation

### ⚙️ Miscellaneous Tasks

- Release 2.0.0-rc.3 version
## [2.0.0-rc.2] - 2025-12-01

### 🚀 Features

- Replace Facade with Store

### 🐛 Bug Fixes

- Tests and dependencies
- Expose OCABundle struct
- Semver for the crate

### 💼 Other

- Adopt to oca-rs 2.0.0
- Expose few more types
- Validation mechanics

### ⚙️ Miscellaneous Tasks

- Update crates
- Use new interface for generating bundle
- Prepare for release
- Remove unused imports and add .cargo to ignore
- Release 2.0.0-rc.2 version
## [2.0.0-rc1] - 2025-04-29

### 🚀 Features

- Update oca to 2.0.0

### ⚙️ Miscellaneous Tasks

- Release 2.0.0-rc1 version
## [0.2.0] - 2025-03-06

### ⚙️ Miscellaneous Tasks

- Update oca dependencies to 0.7.1
- Release 0.2.0 version
## [0.1.5] - 2025-01-28

### ⚙️ Miscellaneous Tasks

- Update oca dependencies to 0.6.10
- Release 0.1.5 version
## [0.1.4] - 2025-01-28

### ⚙️ Miscellaneous Tasks

- Update oca dependencies to 0.6.9
- Release 0.1.4 version
## [0.1.3] - 2025-01-20

### 🐛 Bug Fixes

- Support multiple OCABundle instances in OCABundleInfo cache logic

### ⚙️ Miscellaneous Tasks

- Add initial changelog file
- Release 0.1.3 version
## [0.1.2] - 2025-01-20

### 🚀 Features

- Update BundleInfo to OCABundleInfo to get rid of Bundle and cache info
- Extract links and framings from overlays in OCABundleInfo constructor

### ⚙️ Miscellaneous Tasks

- Update CI workflow to publish on crates.io
- Add configuration for release management
- Release 0.1.2 version
## [0.1.1] - 2025-01-17

### 🚀 Features

- Add basic captured data validation
- Use assets for captured data validation testing
- Add DataValidationStatus as validation Ok result and add OCA semantic validator in test
- Add WithInfo trait and implementation for OCABundle to provide metadata access
- Use validate_semantics function for OCABundle validation
- Rename oca_bundle to structural_bundle and add use load_oca function to load new OCABundle
- Add BundleInfo struct to encapsulate bundle metadata, attributes and links
- Add build_from_ocafile function to the public API
- Validate_data takes serde json value
- Implement ToJSON trait for OCABundle to simplify JSON serialization
- Add parse_oca_bundle_to_ocafile function to the public API

### 🐛 Bug Fixes

- Change return type of attributes() to use impl Iterator

### 💼 Other

- Add CI

### 🚜 Refactor

- Rename load_oca_semantics to load

### 📚 Documentation

- Add docs for #validate_semantics
- Add basic introduction

### ⚙️ Miscellaneous Tasks

- Update README.md
- Create LICENSE
- Update oca-rs and related dependencies to version 0.6.4
- Add license, readme, and description fields to the project metadata
- Update oca dependencies to 0.6.8
