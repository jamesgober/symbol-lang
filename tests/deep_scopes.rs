//! Looking up an outer name from deep inside nested scopes must not cost time in
//! proportion to the nesting depth.
//!
//! The test enters `depth` scopes and looks an outer name up at every level, then
//! compares the fastest of several runs at two depths eight times apart. With a
//! lookup that does not depend on depth, the total grows by roughly 8 to 12 times;
//! a lookup that walks the scope chain makes it grow by about 64 times. Comparing
//! a ratio rather than an absolute time keeps the test stable on a busy machine.

use std::time::{Duration, Instant};

use intern_lang::Interner;
use symbol_lang::SymbolTable;

fn fastest_nested_lookups(depth: usize, runs: usize) -> Duration {
    let mut names = Interner::new();
    let outer = names.intern("outer");
    (0..runs)
        .map(|_| {
            let mut table: SymbolTable<u32> = SymbolTable::new();
            table.define(outer, 1);
            let start = Instant::now();
            for _ in 0..depth {
                table.enter_scope();
                assert_eq!(table.lookup(outer), Some(&1));
            }
            start.elapsed()
        })
        .min()
        .unwrap_or_default()
}

#[test]
fn lookup_cost_does_not_grow_with_nesting() {
    let shallow = fastest_nested_lookups(2_000, 7);
    let deep = fastest_nested_lookups(16_000, 7);
    let ratio = deep.as_nanos() / shallow.as_nanos().max(1);
    assert!(
        ratio < 30,
        "8x deeper nesting took {ratio}x as long ({shallow:?} vs {deep:?})"
    );
}

#[test]
fn deep_shadowing_unwinds_in_order() {
    let mut names = Interner::new();
    let x = names.intern("x");
    let y = names.intern("y");
    let mut table: SymbolTable<usize> = SymbolTable::new();
    table.define(y, 0);
    for level in 1..=1_000 {
        table.enter_scope();
        if level % 3 == 0 {
            assert_eq!(table.define(x, level), None);
            assert_eq!(table.define(x, level + 1), Some(level));
        }
    }
    for level in (1..=1_000).rev() {
        let expect = (1..=level).rev().find(|l| l % 3 == 0).map(|l| l + 1);
        assert_eq!(table.lookup(x).copied(), expect);
        assert_eq!(
            table.lookup_local(x).copied(),
            (level % 3 == 0).then_some(level + 1)
        );
        assert_eq!(table.lookup(y), Some(&0));
        assert!(table.exit_scope());
    }
    assert_eq!(table.lookup(x), None);
    assert_eq!(table.depth(), 1);
}
