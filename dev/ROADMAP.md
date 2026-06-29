# symbol-lang - Roadmap

> Path from scaffold to a stable 1.0. Hard parts are front-loaded; each phase has hard exit criteria.
> Master plan: ../../_strategy/LANG_COLLECTION.md
>
> **Anti-deferral rule:** no listed hard task moves to a later phase unless this file records the move and the reason.

## v0.1.0 - Scaffold (DONE)
Compiles, CI green, structure correct, no domain logic.
- [x] Manifest, README, CHANGELOG, REPS, dual license, CI, deny, clippy, rustfmt.

## v0.2.0 - Core (DONE)
Symbol tables, scope chains, and name binding/resolution.
Dependencies wired when first used: `intern` (`Symbol` keys) and `diag` (name-error
diagnostics; re-exports `Span`). `ast` is not a dependency — name binding is the
language's job, walking its own tree and threading this table; the master plan's
`ast` listing is a use-site relationship. See `dev/NOTES.md`.
Exit criteria:
- [x] Every public item has rustdoc + a runnable example.
- [x] Core invariants property-tested (full DIRECTIVES + API authored at this stage).

## v1.0.0 - API freeze (DONE)
Public surface stable and frozen until 2.0.
- [x] docs/API.md marked stable; SemVer promise recorded.
- [x] Full test + benchmark suite green (Windows local + MSRV; CI runs Linux/macOS/Windows on push).
