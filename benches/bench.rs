//! Benchmarks for the hot paths: defining a name in the current scope, and
//! looking one up through the scope chain.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use intern_lang::{Interner, Symbol};
use std::hint::black_box;
use symbol_lang::SymbolTable;

/// Interns `count` distinct names.
fn names(count: usize) -> (Interner, Vec<Symbol>) {
    let mut interner = Interner::new();
    let syms = (0..count)
        .map(|i| {
            let mut s = String::from("name");
            s.push_str(&i.to_string());
            interner.intern(&s)
        })
        .collect();
    (interner, syms)
}

fn bench_define(c: &mut Criterion) {
    let mut group = c.benchmark_group("define");
    for &count in &[16usize, 256, 4096] {
        let (_i, syms) = names(count);
        group.bench_with_input(BenchmarkId::from_parameter(count), &syms, |b, syms| {
            b.iter(|| {
                let mut table: SymbolTable<u32> = SymbolTable::new();
                for (i, &s) in syms.iter().enumerate() {
                    table.define(black_box(s), i as u32);
                }
                table.depth()
            });
        });
    }
    group.finish();
}

fn bench_lookup_through_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_chain");
    // One name per scope, then look the deepest (outermost) one up from the bottom.
    for &depth in &[4usize, 16, 64] {
        let (_i, syms) = names(depth);
        group.bench_with_input(BenchmarkId::from_parameter(depth), &syms, |b, syms| {
            let mut table: SymbolTable<u32> = SymbolTable::new();
            for (i, &s) in syms.iter().enumerate() {
                table.define(s, i as u32);
                table.enter_scope();
            }
            let target = syms[0]; // defined in the outermost scope
            b.iter(|| table.lookup(black_box(target)).copied());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_define, bench_lookup_through_chain);
criterion_main!(benches);
