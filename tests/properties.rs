//! Property tests: the scope chain is checked against an independent reference
//! model over long random sequences of operations.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use diag_lang::Span;
use intern_lang::{Interner, Symbol};
use proptest::prelude::*;
use symbol_lang::{SymbolTable, unresolved_name};

/// The operations a resolver performs on the table.
#[derive(Clone, Debug)]
enum Op {
    Enter,
    Exit,
    Define(usize, i32),
    Lookup(usize),
    LookupLocal(usize),
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        Just(Op::Enter),
        Just(Op::Exit),
        (0usize..8, any::<i32>()).prop_map(|(n, v)| Op::Define(n, v)),
        (0usize..8).prop_map(Op::Lookup),
        (0usize..8).prop_map(Op::LookupLocal),
    ]
}

proptest! {
    /// The table agrees with a plain `Vec<BTreeMap>` reference model after every
    /// operation: same `define` displacement, same `lookup` and `lookup_local`
    /// result, same depth.
    #[test]
    fn table_matches_reference_model(ops in prop::collection::vec(op_strategy(), 0..300)) {
        let mut interner = Interner::new();
        let syms: Vec<Symbol> = (0..8)
            .map(|i| {
                let mut s = String::from("n");
                s.push_str(&i.to_string());
                interner.intern(&s)
            })
            .collect();

        let mut table: SymbolTable<i32> = SymbolTable::new();
        let mut model: Vec<BTreeMap<Symbol, i32>> = vec![BTreeMap::new()];

        for op in ops {
            match op {
                Op::Enter => {
                    table.enter_scope();
                    model.push(BTreeMap::new());
                }
                Op::Exit => {
                    let popped = table.exit_scope();
                    if model.len() > 1 {
                        model.pop();
                        prop_assert!(popped);
                    } else {
                        prop_assert!(!popped);
                    }
                }
                Op::Define(i, v) => {
                    let s = syms[i];
                    let real = table.define(s, v);
                    let reference = model.last_mut().unwrap().insert(s, v);
                    prop_assert_eq!(real, reference);
                }
                Op::Lookup(i) => {
                    let s = syms[i];
                    let real = table.lookup(s).copied();
                    let reference = model.iter().rev().find_map(|m| m.get(&s).copied());
                    prop_assert_eq!(real, reference);
                }
                Op::LookupLocal(i) => {
                    let s = syms[i];
                    let real = table.lookup_local(s).copied();
                    let reference = model.last().and_then(|m| m.get(&s).copied());
                    prop_assert_eq!(real, reference);
                }
            }
            prop_assert_eq!(table.depth(), model.len());
        }
    }

    /// `scoped` isolates a balanced block: definitions made inside vanish with the
    /// scope, and the depth is the same before and after.
    #[test]
    fn scoped_isolates_its_definitions(defines in prop::collection::vec(0usize..8, 0..20)) {
        let mut interner = Interner::new();
        let syms: Vec<Symbol> = (0..8)
            .map(|i| {
                let mut s = String::from("v");
                s.push_str(&i.to_string());
                interner.intern(&s)
            })
            .collect();

        let mut table: SymbolTable<i32> = SymbolTable::new();
        let before = table.depth();
        table.scoped(|table| {
            for &i in &defines {
                table.define(syms[i], i as i32);
            }
        });

        // The single entered scope was exited, and everything it defined is gone
        // (nothing was defined at the root).
        prop_assert_eq!(table.depth(), before);
        for s in &syms {
            prop_assert!(table.lookup(*s).is_none());
        }
        let _ = unresolved_name("x", Span::new(0, 1)); // smoke: diag helper is callable
    }
}
