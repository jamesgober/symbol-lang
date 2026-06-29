# symbol-lang &mdash; Engineering Directives

> Engineering standards and the definition of done for this project. Read alongside `REPS.md` (root, authoritative) and `dev/ROADMAP.md` (current phase). If anything here conflicts with `REPS.md`, `REPS.md` wins.

---

## 0. Philosophy

This library is built and maintained to a production standard and treated as a flagship piece of work. Plan the full path, then build one verified step at a time. "Good enough" is treated as a defect. symbol-lang is the first SEMA-tier crate: name resolution is where a tree of syntax becomes a program with meaning, so its scoping must be exactly right — a name must resolve to the binding a reader would expect, shadowing and all — and its operations must be total on any sequence of calls.

---

## 1. What this is

symbol-lang is the name-resolution substrate: a lexically-scoped [`SymbolTable`](SymbolTable) that maps interned names to bindings, plus ready-made [`diag-lang`] diagnostics for the two name errors every language reports — a name used but not defined, and a name defined twice. It is generic over the binding — the language decides what a name resolves *to* (a node handle, a type, a declaration record) — and keys every scope on an [`intern-lang`] `Symbol`. It owns scopes and lookup only — no AST walk (the language drives binding over its own tree, the use-site relationship the master plan calls `ast`), no type information, no lexing.

The model is the lexical scope chain a single-pass resolver uses: enter a scope on the way into a block, define names in it, look names up outward through the enclosing scopes, and exit the scope on the way out. A name resolves to the nearest enclosing binding, so an inner definition shadows an outer one for as long as the inner scope is open.

---

## 2. Engineering law (non-negotiable)

- **Performance** — `define` and `lookup_local` are a single map operation on the current scope; `lookup` walks outward at most as deep as the scope chain. Scope maps are ordered by the four-byte `Symbol`, so a comparison is an integer compare. No "faster" claim without `criterion` numbers.
- **Correctness** — lookup obeys lexical scoping exactly: it returns the nearest enclosing binding, an inner definition shadows an outer one while its scope is open, and the outer one is visible again after the inner scope exits. The invariants in section 4 are property-tested against a reference scope model.
- **Robustness** — every operation is total: looking up an unbound name is `None`, exiting the root scope is a defined no-op, and no sequence of calls panics or corrupts the chain.
- **Architecture** — SOLID, KISS, YAGNI; one responsibility; the table is generic over the binding, so it depends on no concrete language or AST.
- **Cross-platform** — Linux/macOS/Windows first-class; no platform-specific code.
- **Error handling** — name errors are values: `define` reports a clash by returning the displaced binding, `lookup` reports an absent name as `None`, and the diagnostic helpers turn those into `diag-lang` diagnostics. No panics in library code.
- **Production-ready** — `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` from the first commit; no stray `println!`/`dbg!`; every public item has rustdoc with a runnable example.

---

## 3. Definition of done

1. Compiles clean on Linux/macOS/Windows, stable and MSRV 1.85.
2. `fmt`, `clippy -D warnings`, `test --all-features`, `cargo doc -D warnings` clean.
3. `cargo audit` + `cargo deny check` pass.
4. No `unwrap`/`expect`/`todo!`/`dbg!` in shipping code.
5. A Tier-1 API exists and headlines the docs.
6. Property tests cover the section-4 scoping invariants.
7. Hot-path (`define`/`lookup`) changes carry benchmarks; no regression over 5%.
8. Docs and `CHANGELOG.md` updated; the matching `docs/release/vX.Y.Z.md` written before the tag.

---

## 4. Project-specific invariants

- `lookup` returns the binding from the nearest scope that defines the name, searching the current scope first and the root last; an unbound name is `None`.
- A definition in an inner scope shadows the same name in an outer scope for exactly as long as the inner scope is open; after the inner scope exits, the outer binding is visible again.
- `define` overwrites and returns any binding the same name already had *in the current scope*, so a clash is detectable; it never affects an enclosing scope.
- The scope chain is always non-empty: there is a root scope that cannot be exited, so `depth` is at least one and the table is usable immediately after construction.
- `scoped` enters exactly one scope, runs the closure, and exits it, so the chain depth is the same before and after regardless of what the closure does.
- The diagnostic helpers produce a stable, documented form: an unresolved name points a primary label at the reference; a duplicate definition points a primary at the redefinition and a secondary at the first definition.
