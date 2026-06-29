# symbol-lang — Engineering Notes

Decisions taken while building symbol-lang to 1.0, recorded for review. Lives
alongside `DIRECTIVES.md` and `ROADMAP.md` in `dev/`.

---

## Design decisions

### Generic over the binding; no `ast-lang` dependency

The master plan lists symbol-lang's dependencies as `ast, diag, intern`. symbol-lang
wires **`intern`** (names are `Symbol`s, the scope keys) and **`diag`** (the
name-error diagnostics; it also re-exports `Span` from there, so no direct
span-lang dependency is needed), but **not `ast`**.

A `SymbolTable<T>` is generic over the binding `T` — the language decides what a
name resolves *to* (an `ast-lang` node handle, a declaration record, a type). Name
*binding* itself — which nodes introduce names and which reference them — is
inherently language-specific: the language walks its own tree (with `ast-lang`) and
threads the table. So `ast` is a use-site relationship, not an import, the same call
made in parser-lang. Per the build directive's "wire a dependency only when code
actually uses it," `ast-lang` was not added. It can be added as a non-breaking minor
if a generic AST-aware helper ever earns its place.

### A scope *stack*, not a persistent tree

The table is a single-pass scope stack: `exit_scope` discards a scope's bindings.
That is exactly what a resolver needs walking a tree once and annotating it. A
*persistent* scope tree (scopes retained and addressable, for IDE "what is in scope
at position X" queries) is a different, larger structure; it can be added later
without breaking this surface.

### Diagnostics take the name as `&str`

`unresolved_name` / `duplicate_definition` take the human-readable name, not a
`Symbol`, because rendering the name requires the interner — which the table does not
own. The caller resolves `Symbol → &str` with its interner and passes spans it has
from the source. This keeps symbol-lang free of an interner dependency at the
diagnostic boundary and the table generic.

### `BTreeMap` per scope (`no_std`)

Scopes are `BTreeMap<Symbol, T>`, so the crate is `no_std` + `alloc` with no
hashing dependency. `Symbol` is a four-byte `Ord` key, so a comparison is an integer
compare; scopes are small, so the `O(log n)` lookup is well within budget.

---

## STATUS — 1.0.0, frozen

symbol-lang is built from scaffold through the v0.2.0 core to the v1.0.0 API freeze.
Engineering is complete and green; the commit, tag (`v1.0.0`), push, and
`cargo publish` are left for you, as requested.

**Done:**

- v0.2.0 core — `SymbolTable<T>` (the lexical scope chain) and the
  `unresolved_name` / `duplicate_definition` diagnostics. `intern-lang` and
  `diag-lang` wired and used; `Symbol`/`Diagnostic`/`Span` re-exported.
- v1.0.0 freeze — `docs/API.md` documents the frozen surface and the SemVer promise.

**Quality gates, all green** (Windows local via the rust-lld linker workaround; CI
runs the same on Linux/macOS/Windows, stable + 1.85): `fmt --check`, `clippy
-D warnings` on default / all-features / no-default-features, `test` on default and
all-features, `doc -D warnings`, no-default-features (`no_std`) build, `+1.85`
build, `publish --dry-run`, `deny check`. `#![forbid(unsafe_code)]`, no
`unwrap`/`expect`/`todo!` in shipping code. Counts: 10 unit + 4 resolution + 2
property + 13 doctests.

**For your review before tagging:** the deviation above (no `ast` dependency;
generic over the binding). It is defensible and documented; flag it if it disagrees
with the master plan's intent for symbol-lang.
