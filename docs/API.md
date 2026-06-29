# symbol-lang &mdash; API Reference

> Complete reference for every public item in `symbol-lang`, with examples.
> **Status: stable as of `1.0.0`.** The surface below is frozen under Semantic Versioning — no breaking changes before `2.0`.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [`SymbolTable`](#symboltable)
- [Diagnostics](#diagnostics)
- [Re-exports](#re-exports)
- [Feature flags](#feature-flags)
- [Frozen surface](#frozen-surface)

---

## Overview

symbol-lang is the name-resolution substrate of the `-lang` family: a
lexically-scoped [`SymbolTable`](#symboltable) mapping interned names to bindings,
plus ready-made diagnostics for the two name errors every language reports. It is
generic over the binding — the language decides what a name resolves *to* — and
keys every scope on an [`intern-lang`](https://docs.rs/intern-lang) `Symbol`, so a
lookup compares integers.

A resolver threads the table through its own tree walk: enter a scope per block,
define the names it introduces, look references up outward, exit on the way out. A
name resolves to the nearest enclosing binding; an inner definition shadows an
outer one while its scope is open.

---

## Installation

```toml
[dependencies]
symbol-lang = "1"
```

---

## `SymbolTable`

`SymbolTable<T>` is the scope chain — a stack of scopes, each a map from `Symbol`
to a binding `T`. There is always a root scope, so it is usable immediately and the
root can never be exited.

```rust
use symbol_lang::SymbolTable;
use intern_lang::Interner;

let mut names = Interner::new();
let x = names.intern("x");

let mut table: SymbolTable<i32> = SymbolTable::new();
table.define(x, 1);
table.enter_scope();
table.define(x, 2);                   // shadows
assert_eq!(table.lookup(x), Some(&2));
table.exit_scope();
assert_eq!(table.lookup(x), Some(&1));
```

| Method | Description |
|--------|-------------|
| `new() -> SymbolTable<T>` | A table with a single empty root scope. |
| `enter_scope(&mut self)` | Push a new innermost scope. |
| `exit_scope(&mut self) -> bool` | Pop the innermost scope; a no-op returning `false` at the root. |
| `scoped<R>(&mut self, f) -> R` | Run `f` in a freshly entered scope, exiting it after. |
| `define(&mut self, name: Symbol, value: T) -> Option<T>` | Bind in the current scope; returns any binding it replaces there. |
| `lookup(&self, name: Symbol) -> Option<&T>` | Find the nearest enclosing binding. |
| `lookup_local(&self, name: Symbol) -> Option<&T>` | Find in the current scope only. |
| `is_defined(&self, name: Symbol) -> bool` | Whether the name is bound anywhere in the chain. |
| `depth(&self) -> usize` | Number of active scopes (always ≥ 1). |

`SymbolTable<T>` implements `Default` (= `new`). The binding `T` has no trait
bounds.

---

## Diagnostics

Free functions that build the two universal name-error [`diag-lang`](https://docs.rs/diag-lang)
diagnostics. `name` is the human-readable spelling, which the caller resolves from
the `Symbol` with its interner; the spans come from the source.

| Function | Description |
|----------|-------------|
| `unresolved_name(name: &str, reference: Span) -> Diagnostic` | "cannot find `{name}` in this scope", a primary label at the reference. |
| `duplicate_definition(name: &str, redefinition: Span, first: Span) -> Diagnostic` | "the name `{name}` is defined multiple times", a primary at the redefinition and a secondary at the first definition. |

```rust
use symbol_lang::{duplicate_definition, unresolved_name};
use diag_lang::Span;

let a = unresolved_name("foo", Span::new(8, 11));
assert_eq!(a.message(), "cannot find `foo` in this scope");

let b = duplicate_definition("x", Span::new(30, 31), Span::new(4, 5));
assert_eq!(b.secondary().len(), 1);
```

---

## Re-exports

The key, span, and error types this crate's API uses are re-exported, so a resolver
need not also name `intern-lang` and `diag-lang`:

- `Symbol` — from [`intern-lang`](https://docs.rs/intern-lang); the name key.
- `Diagnostic`, `Span` — from [`diag-lang`](https://docs.rs/diag-lang); the diagnostics and the spans they carry.

---

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Standard library. With it off, the crate is `no_std` (it always needs `alloc`). Forwards to `diag-lang/std` and `intern-lang/std`. |

---

## Frozen surface

The items above are the complete public API, frozen as of `1.0.0`, under Semantic
Versioning: no breaking change before `2.0`, additions in minors, MSRV (Rust 1.85)
rising only in a minor.

Deliberately left out of 1.0, each addable later without a breaking change:

- **No `ast-lang` dependency.** Name *binding* — deciding which AST nodes introduce
  and reference names — is the language's job, walking its own tree (with
  `ast-lang`) and threading this table. So `ast` is a use-site relationship, not an
  import; the master plan lists it for the pipeline, not as a dependency. See
  `dev/NOTES.md`.
- **No persistent scope tree.** The table is a single-pass scope *stack* (scopes
  are discarded on exit), which is what a resolver needs. A retained scope tree for
  later IDE-style "what is in scope at position X" queries can be added later.
- **No built-in interner.** Names are `Symbol`s the caller interns; symbol-lang
  stores and compares them but does not own an interner.

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>
