<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>symbol-lang</b>
    <br>
    <sub><sup>SYMBOLS & SCOPES</sup></sub>
</h1>

<div align="center">
    <a href="https://crates.io/crates/symbol-lang"><img alt="Crates.io" src="https://img.shields.io/crates/v/symbol-lang"></a>
    <a href="https://crates.io/crates/symbol-lang"><img alt="Downloads" src="https://img.shields.io/crates/d/symbol-lang?color=%230099ff"></a>
    <a href="https://docs.rs/symbol-lang"><img alt="docs.rs" src="https://img.shields.io/docsrs/symbol-lang"></a>
    <a href="https://github.com/jamesgober/symbol-lang/actions"><img alt="CI" src="https://github.com/jamesgober/symbol-lang/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        symbol-lang is the SEMA-tier crate: Symbol tables, scope chains, and name binding/resolution over the AST. Part of the -lang language-construction family; see _strategy/LANG_COLLECTION.md for the master plan.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.85+</strong> (Rust 2024 edition).
    </p>
    <blockquote>
        <strong>Status: stable.</strong> The public API is frozen as of <code>1.0.0</code> and follows Semantic Versioning &mdash; no breaking changes before <code>2.0</code>. See <a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

## Installation

```toml
[dependencies]
symbol-lang = "1"
```

<br>

## Example

A lexically-scoped symbol table generic over the binding. A name resolves to the nearest enclosing definition; inner scopes shadow.

```rust
use symbol_lang::{unresolved_name, SymbolTable};
use intern_lang::Interner;
use diag_lang::Span;

let mut names = Interner::new();
let x = names.intern("x");
let y = names.intern("y");

let mut table: SymbolTable<i32> = SymbolTable::new();
table.define(x, 1);

table.enter_scope();
table.define(x, 2);                       // shadows the outer `x`
assert_eq!(table.lookup(x), Some(&2));
table.exit_scope();
assert_eq!(table.lookup(x), Some(&1));     // outer `x` visible again

// An unresolved reference becomes a diagnostic.
assert!(table.lookup(y).is_none());
let diag = unresolved_name("y", Span::new(10, 11));
assert_eq!(diag.message(), "cannot find `y` in this scope");
```

<br>

## Status

This is <code>v1.0.0</code>: the public API is stable and frozen under SemVer. The lexically-scoped <code>SymbolTable</code> and the name-error diagnostics are complete and catalogued in <a href="./docs/API.md"><code>docs/API.md</code></a>.

<hr>
<br>

## Contributing

See <a href="./dev/DIRECTIVES.md"><code>dev/DIRECTIVES.md</code></a> for engineering standards and the definition of done. Before a PR: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` must be clean.

<br>

<div id="license">
    <h2>License</h2>
    <p>Licensed under either of</p>
    <ul>
        <li><b>Apache License, Version 2.0</b> &mdash; <a href="./LICENSE-APACHE">LICENSE-APACHE</a></li>
        <li><b>MIT License</b> &mdash; <a href="./LICENSE-MIT">LICENSE-MIT</a></li>
    </ul>
    <p>at your option.</p>
</div>

<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>James Gober <me@jamesgober.com>.</strong></sup>
</div>
