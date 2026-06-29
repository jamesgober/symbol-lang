<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>CHANGELOG</b>
</h1>
<p>
  All notable changes to <code>symbol-lang</code> will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

---

## [Unreleased]

### Added

### Changed

### Fixed

### Security

---

## [1.0.0] - 2026-06-28

API freeze. The symbol-table and diagnostics surface is stable and will not change
in a breaking way before `2.0`. No functional API changes from `0.2.0`.

### Changed

- The public API is declared stable under Semantic Versioning; `docs/API.md`
  catalogues the frozen surface and the SemVer promise.

---

## [0.2.0] - 2026-06-28

The core: the lexically-scoped symbol table and the name-error diagnostics.
`intern-lang` and `diag-lang` are wired and used.

### Added

- `SymbolTable<T>` — a lexically-scoped name→binding map, generic over the binding: `new`, `enter_scope`, `exit_scope`, `scoped`, `define`, `lookup`, `lookup_local`, `is_defined`, `depth`. A name resolves to the nearest enclosing binding; inner scopes shadow.
- `unresolved_name` and `duplicate_definition` — ready-made `diag-lang` diagnostics for a name used but not defined, and a name defined twice (with both sites labelled).
- `Symbol` (from `intern-lang`) and `Diagnostic` / `Span` (from `diag-lang`) re-exported from the crate root.
- An end-to-end resolution test (binding, shadowing, unresolved, duplicate) and a property test checking the scope chain against an independent reference model; benchmarks for `define` and chained `lookup`.

### Removed

- The reserved no-op `serde` feature, dropped rather than carried unused toward the frozen 1.0 surface.

---

## [0.1.0] - 2026-06-18

Initial scaffold and repository bootstrap. No domain logic yet &mdash; this release establishes the structure, tooling, and quality gates the implementation will be built on.

### Added

- `Cargo.toml` with crate metadata, Rust 2024 edition, MSRV 1.85.
- Dual `Apache-2.0 OR MIT` license files.
- `README.md`, `CHANGELOG.md`, and a documentation skeleton.
- `REPS.md` compliance baseline.
- `.github/workflows/ci.yml` CI matrix; `deny.toml`, `clippy.toml`, `rustfmt.toml`.
- `dev/DIRECTIVES.md` and `dev/ROADMAP.md` (committed engineering standards + plan).

[Unreleased]: https://github.com/jamesgober/symbol-lang/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/jamesgober/symbol-lang/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/jamesgober/symbol-lang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/symbol-lang/releases/tag/v0.1.0
