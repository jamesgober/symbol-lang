//! An end-to-end name-resolution pass over a tiny program, showing the intended
//! use: thread a `SymbolTable` through a tree walk, binding definitions and
//! resolving references, and turn the failures into diagnostics.

#![allow(clippy::unwrap_used)]

use diag_lang::{Diagnostic, Span};
use intern_lang::{Interner, Symbol};
use symbol_lang::{SymbolTable, duplicate_definition, unresolved_name};

/// A flattened "program": a sequence of events a resolver would encounter walking
/// a tree. Each carries the span it occupies in the (imagined) source.
enum Event {
    Define(Symbol, Span),
    Use(Symbol, Span),
    EnterBlock,
    ExitBlock,
}

/// Resolves a program, returning the diagnostics it produces. A definition binds
/// the name to the span it was defined at; a duplicate in the same scope and an
/// unresolved use are reported.
fn resolve(events: &[Event], interner: &Interner) -> Vec<Diagnostic> {
    let mut table: SymbolTable<Span> = SymbolTable::new();
    let mut diagnostics = Vec::new();

    for event in events {
        match event {
            Event::Define(name, span) => {
                if let Some(&first) = table.lookup_local(*name) {
                    let text = interner.resolve(*name).unwrap_or("?");
                    diagnostics.push(duplicate_definition(text, *span, first));
                }
                table.define(*name, *span);
            }
            Event::Use(name, span) => {
                if table.lookup(*name).is_none() {
                    let text = interner.resolve(*name).unwrap_or("?");
                    diagnostics.push(unresolved_name(text, *span));
                }
            }
            Event::EnterBlock => table.enter_scope(),
            Event::ExitBlock => {
                table.exit_scope();
            }
        }
    }

    diagnostics
}

#[test]
fn test_well_formed_program_has_no_diagnostics() {
    let mut interner = Interner::new();
    let x = interner.intern("x");
    let y = interner.intern("y");

    // let x; { let y; use x; use y; }
    let program = [
        Event::Define(x, Span::new(0, 1)),
        Event::EnterBlock,
        Event::Define(y, Span::new(2, 3)),
        Event::Use(x, Span::new(4, 5)), // resolves to the outer `x`
        Event::Use(y, Span::new(6, 7)), // resolves to the inner `y`
        Event::ExitBlock,
    ];

    let diagnostics = resolve(&program, &interner);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_use_after_scope_exit_is_unresolved() {
    let mut interner = Interner::new();
    let y = interner.intern("y");

    // { let y; } use y;   — `y` is out of scope at the use site.
    let program = [
        Event::EnterBlock,
        Event::Define(y, Span::new(0, 1)),
        Event::ExitBlock,
        Event::Use(y, Span::new(5, 6)),
    ];

    let diagnostics = resolve(&program, &interner);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message(), "cannot find `y` in this scope");
    assert_eq!(diagnostics[0].primary().span(), Span::new(5, 6));
}

#[test]
fn test_duplicate_in_same_scope_is_reported_with_both_sites() {
    let mut interner = Interner::new();
    let x = interner.intern("x");

    // let x; let x;   — redefined in the same scope.
    let program = [
        Event::Define(x, Span::new(0, 1)),
        Event::Define(x, Span::new(8, 9)),
    ];

    let diagnostics = resolve(&program, &interner);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message(),
        "the name `x` is defined multiple times"
    );
    assert_eq!(diagnostics[0].primary().span(), Span::new(8, 9));
    assert_eq!(diagnostics[0].secondary()[0].span(), Span::new(0, 1));
}

#[test]
fn test_shadowing_in_inner_scope_is_not_a_duplicate() {
    let mut interner = Interner::new();
    let x = interner.intern("x");

    // let x; { let x; }   — the inner `x` shadows; not a duplicate.
    let program = [
        Event::Define(x, Span::new(0, 1)),
        Event::EnterBlock,
        Event::Define(x, Span::new(4, 5)),
        Event::ExitBlock,
    ];

    let diagnostics = resolve(&program, &interner);
    assert!(diagnostics.is_empty());
}
