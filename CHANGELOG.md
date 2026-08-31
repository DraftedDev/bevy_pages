# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/DraftedDev/bevy_pages/compare/v0.3.5...v0.4.0) - 2026-08-31

### Added

- Add Multi Page Docs
- Add With<ElementActive> Condition to Widget Logic
- Add multi-page example
- [**breaking**] Rework Page Spawner to Page Manager

### Fixed

- Glitched initial text input width

### Other

- Update README.md

## [0.3.5](https://github.com/DraftedDev/bevy_pages/compare/v0.3.4...v0.3.5) - 2026-08-27

### Other

- Optimize ElementId PartialEq
- Rework Widget::apply_defaults to use AttributeMap
- Load via Vec<u8> instead of String
- Create release-plz.yml
- Update package description
- Update quick start with new counter example
- Rework counter example to use styles
- Fix button widget default attributes
- Update widget-dev.md
- Replace String with SmolStr where possible
- Update README.md
- Remove `ElementRegistry` in favor of direct `FxHashMap`
