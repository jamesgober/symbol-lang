//! # symbol_lang
//!
//! The name-resolution substrate for the `-lang` family: a lexically-scoped
//! [`SymbolTable`] that maps interned names to bindings, and ready-made
//! diagnostics for the two name errors every language reports. It is generic over
//! the binding — the language decides what a name resolves *to* — so one scope
//! engine serves every language built on it.
//!
//! ## Model
//!
//! A resolver walks its own syntax tree and threads a [`SymbolTable`]: it enters a
//! scope on the way into a block, [`define`](SymbolTable::define)s the names that
//! block introduces, [`lookup`](SymbolTable::lookup)s references outward through the
//! enclosing scopes, and exits the scope on the way out (or wraps the block in
//! [`scoped`](SymbolTable::scoped)). A name resolves to the nearest enclosing
//! binding, so an inner definition shadows an outer one while its scope is open.
//! Names are [`intern_lang`] [`Symbol`]s, so a lookup compares integers.
//!
//! When a name does not resolve, or is defined twice, [`unresolved_name`] and
//! [`duplicate_definition`] turn that into a source-annotated [`diag_lang`]
//! [`Diagnostic`].
//!
//! ## Quickstart
//!
//! ```
//! use symbol_lang::{duplicate_definition, unresolved_name, SymbolTable};
//! use intern_lang::Interner;
//! use diag_lang::Span;
//!
//! let mut names = Interner::new();
//! let x = names.intern("x");
//! let y = names.intern("y");
//!
//! // Bindings can be anything — here, the span where each name was defined.
//! let mut table: SymbolTable<Span> = SymbolTable::new();
//! table.define(x, Span::new(0, 1));
//!
//! // Resolve a reference.
//! assert!(table.lookup(x).is_some());
//! assert!(table.lookup(y).is_none());
//!
//! // Turn an unresolved reference into a diagnostic.
//! let diag = unresolved_name("y", Span::new(10, 11));
//! assert_eq!(diag.message(), "cannot find `y` in this scope");
//!
//! // A duplicate definition links both sites.
//! if let Some(first) = table.lookup_local(x).copied() {
//!     let _ = duplicate_definition("x", Span::new(20, 21), first);
//! }
//! ```
//!
//! ## Features
//!
//! - `std` (default) — the standard library; without it the crate is `no_std`
//!   (it always needs `alloc`). Forwards to `diag-lang/std` and `intern-lang/std`.
//!
//! ## Stability
//!
//! The public surface is being designed across the 0.x series and freezes at
//! `1.0.0`, after which it follows Semantic Versioning: no breaking changes before
//! `2.0`, additions arrive in minor releases, and the MSRV (Rust 1.85) only rises
//! in a minor. The frozen surface is catalogued in
//! [`docs/API.md`](https://github.com/jamesgober/symbol-lang/blob/main/docs/API.md).

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

extern crate alloc;

mod diagnostics;
mod table;

pub use diagnostics::{duplicate_definition, unresolved_name};
pub use table::SymbolTable;

// Re-exported so a resolver can name the key, span, and error types this crate's
// API uses without also having to depend on `intern-lang` and `diag-lang`.
pub use diag_lang::{Diagnostic, Span};
pub use intern_lang::Symbol;
