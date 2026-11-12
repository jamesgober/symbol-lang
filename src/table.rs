//! The lexically-scoped symbol table.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use intern_lang::Symbol;

/// A lexically-scoped map from interned names to bindings.
///
/// `SymbolTable<T>` is the scope chain a name resolver threads: a stack of scopes,
/// each mapping a [`Symbol`] to a binding of the language's choosing — a node
/// handle, a declaration record, a type, whatever the language resolves a name
/// *to*. The table is generic over that binding `T` and keys every scope on the
/// four-byte `Symbol`, so a lookup compares integers, not strings.
///
/// A resolver [`enter_scope`](SymbolTable::enter_scope) on the way into a block,
/// [`define`](SymbolTable::define)s names in it, [`lookup`](SymbolTable::lookup)s
/// names outward through the enclosing scopes, and
/// [`exit_scope`](SymbolTable::exit_scope) on the way out — or wraps the whole
/// block in [`scoped`](SymbolTable::scoped). A name resolves to the nearest
/// enclosing binding, so an inner definition shadows an outer one while its scope
/// is open.
///
/// There is always a root scope, so the table is usable the moment it is built and
/// the root can never be exited.
///
/// # Examples
///
/// ```
/// use symbol_lang::SymbolTable;
/// use intern_lang::Interner;
///
/// let mut names = Interner::new();
/// let x = names.intern("x");
///
/// let mut table: SymbolTable<i32> = SymbolTable::new();
/// table.define(x, 1); // outer `x` = 1
/// assert_eq!(table.lookup(x), Some(&1));
///
/// table.enter_scope();
/// table.define(x, 2); // inner `x` shadows the outer one
/// assert_eq!(table.lookup(x), Some(&2));
/// table.exit_scope();
///
/// assert_eq!(table.lookup(x), Some(&1)); // outer `x` is visible again
/// ```
pub struct SymbolTable<T> {
    /// Every live binding, keyed by name. Each name maps to a stack of
    /// `(scope index, value)` pairs, innermost last, so the nearest binding is
    /// always on top and a lookup costs one map search whatever the depth.
    bindings: BTreeMap<Symbol, Vec<(usize, T)>>,
    /// The names defined in each open scope, outermost first. Never empty:
    /// `scopes[0]` is the root, and [`exit_scope`](SymbolTable::exit_scope)
    /// refuses to pop it. Exiting a scope pops exactly these names' stacks.
    scopes: Vec<Vec<Symbol>>,
}

impl<T> SymbolTable<T> {
    /// Creates a table with a single, empty root scope.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    ///
    /// let table: SymbolTable<()> = SymbolTable::new();
    /// assert_eq!(table.depth(), 1);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
            scopes: alloc::vec![Vec::new()],
        }
    }

    /// Index of the current (innermost) scope. The chain is never empty.
    fn current(&self) -> usize {
        self.scopes.len() - 1 // `len >= 1` by the type invariant
    }

    /// Pushes a new, empty innermost scope.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    ///
    /// let mut table: SymbolTable<()> = SymbolTable::new();
    /// table.enter_scope();
    /// assert_eq!(table.depth(), 2);
    /// ```
    pub fn enter_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    /// Pops the innermost scope, discarding its bindings, and returns `true`.
    ///
    /// Exiting the root scope is a no-op that returns `false`, so the chain always
    /// keeps at least one scope.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    ///
    /// let mut table: SymbolTable<()> = SymbolTable::new();
    /// table.enter_scope();
    /// assert!(table.exit_scope()); // popped the inner scope
    /// assert!(!table.exit_scope()); // root cannot be exited
    /// assert_eq!(table.depth(), 1);
    /// ```
    pub fn exit_scope(&mut self) -> bool {
        if self.scopes.len() <= 1 {
            return false;
        }
        let Some(names) = self.scopes.pop() else {
            return false;
        };
        for name in names {
            if let Some(stack) = self.bindings.get_mut(&name) {
                // The innermost binding of every name defined in this scope is the
                // one this scope made, so popping the top removes exactly it.
                let _ = stack.pop();
                if stack.is_empty() {
                    let _ = self.bindings.remove(&name);
                }
            }
        }
        true
    }

    /// Runs `f` inside a freshly entered scope, exiting it afterward, and returns
    /// what `f` returns.
    ///
    /// This is the balanced way to scope a block: exactly one scope is entered and
    /// exited, so the chain depth is the same before and after no matter what `f`
    /// does.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    /// use intern_lang::Interner;
    ///
    /// let mut names = Interner::new();
    /// let y = names.intern("y");
    /// let mut table: SymbolTable<i32> = SymbolTable::new();
    ///
    /// let seen = table.scoped(|table| {
    ///     table.define(y, 9);
    ///     table.lookup(y).copied()
    /// });
    /// assert_eq!(seen, Some(9));
    /// assert_eq!(table.lookup(y), None); // the inner `y` is gone
    /// assert_eq!(table.depth(), 1);
    /// ```
    pub fn scoped<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.enter_scope();
        let result = f(self);
        let _ = self.exit_scope();
        result
    }

    /// Binds `name` to `value` in the current scope, returning any binding the same
    /// name already had *in that scope* (which it replaces).
    ///
    /// A `Some` return means the name was already defined in the current scope — a
    /// duplicate definition the caller can report. It never touches an enclosing
    /// scope, so it cannot disturb a shadowed outer binding.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    /// use intern_lang::Interner;
    ///
    /// let mut names = Interner::new();
    /// let n = names.intern("n");
    /// let mut table: SymbolTable<i32> = SymbolTable::new();
    ///
    /// assert_eq!(table.define(n, 1), None); // fresh
    /// assert_eq!(table.define(n, 2), Some(1)); // redefined: the old binding comes back
    /// ```
    pub fn define(&mut self, name: Symbol, value: T) -> Option<T> {
        let current = self.current();
        let stack = self.bindings.entry(name).or_default();
        if let Some((scope, existing)) = stack.last_mut() {
            if *scope == current {
                return Some(core::mem::replace(existing, value));
            }
        }
        stack.push((current, value));
        self.scopes[current].push(name);
        None
    }

    /// Looks `name` up through the scope chain, returning the binding from the
    /// nearest scope that defines it, or `None` if no scope does.
    ///
    /// The cost is one map search over the names currently bound, whatever the
    /// nesting depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    /// use intern_lang::Interner;
    ///
    /// let mut names = Interner::new();
    /// let g = names.intern("g");
    /// let mut table: SymbolTable<&str> = SymbolTable::new();
    /// table.define(g, "global");
    /// table.enter_scope();
    /// assert_eq!(table.lookup(g), Some(&"global")); // found in the outer scope
    /// ```
    #[must_use]
    pub fn lookup(&self, name: Symbol) -> Option<&T> {
        self.bindings
            .get(&name)
            .and_then(|stack| stack.last())
            .map(|(_, value)| value)
    }

    /// Looks `name` up in the current scope only, ignoring enclosing scopes.
    ///
    /// Useful for a duplicate-in-this-scope check before defining, when shadowing an
    /// outer binding is allowed but redefining in the same scope is not.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    /// use intern_lang::Interner;
    ///
    /// let mut names = Interner::new();
    /// let v = names.intern("v");
    /// let mut table: SymbolTable<i32> = SymbolTable::new();
    /// table.define(v, 1);
    /// table.enter_scope();
    /// assert_eq!(table.lookup_local(v), None); // not in *this* scope
    /// assert_eq!(table.lookup(v), Some(&1)); // but visible through the chain
    /// ```
    #[must_use]
    pub fn lookup_local(&self, name: Symbol) -> Option<&T> {
        let current = self.current();
        self.bindings
            .get(&name)
            .and_then(|stack| stack.last())
            .filter(|(scope, _)| *scope == current)
            .map(|(_, value)| value)
    }

    /// Returns `true` if `name` is bound anywhere in the scope chain.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    /// use intern_lang::Interner;
    ///
    /// let mut names = Interner::new();
    /// let a = names.intern("a");
    /// let mut table: SymbolTable<()> = SymbolTable::new();
    /// assert!(!table.is_defined(a));
    /// table.define(a, ());
    /// assert!(table.is_defined(a));
    /// ```
    #[must_use]
    pub fn is_defined(&self, name: Symbol) -> bool {
        self.lookup(name).is_some()
    }

    /// Returns the number of active scopes, including the root — always at least 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use symbol_lang::SymbolTable;
    ///
    /// let mut table: SymbolTable<()> = SymbolTable::new();
    /// assert_eq!(table.depth(), 1);
    /// table.enter_scope();
    /// assert_eq!(table.depth(), 2);
    /// ```
    #[must_use]
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }
}

impl<T> Default for SymbolTable<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use intern_lang::Interner;

    use super::*;

    /// Interns three distinct names for tests.
    fn names() -> (Interner, Symbol, Symbol, Symbol) {
        let mut interner = Interner::new();
        let a = interner.intern("a");
        let b = interner.intern("b");
        let c = interner.intern("c");
        (interner, a, b, c)
    }

    #[test]
    fn test_new_has_one_root_scope() {
        let table: SymbolTable<()> = SymbolTable::new();
        assert_eq!(table.depth(), 1);
    }

    #[test]
    fn test_define_then_lookup() {
        let (_i, a, _b, _c) = names();
        let mut table: SymbolTable<i32> = SymbolTable::new();
        assert_eq!(table.define(a, 7), None);
        assert_eq!(table.lookup(a), Some(&7));
    }

    #[test]
    fn test_redefine_returns_previous() {
        let (_i, a, _b, _c) = names();
        let mut table: SymbolTable<i32> = SymbolTable::new();
        table.define(a, 1);
        assert_eq!(table.define(a, 2), Some(1));
        assert_eq!(table.lookup(a), Some(&2));
    }

    #[test]
    fn test_inner_scope_shadows_then_restores() {
        let (_i, a, _b, _c) = names();
        let mut table: SymbolTable<i32> = SymbolTable::new();
        table.define(a, 1);
        table.enter_scope();
        table.define(a, 2);
        assert_eq!(table.lookup(a), Some(&2));
        assert_eq!(table.lookup_local(a), Some(&2));
        table.exit_scope();
        assert_eq!(table.lookup(a), Some(&1));
    }

    #[test]
    fn test_lookup_local_ignores_outer() {
        let (_i, a, _b, _c) = names();
        let mut table: SymbolTable<i32> = SymbolTable::new();
        table.define(a, 1);
        table.enter_scope();
        assert_eq!(table.lookup_local(a), None);
        assert_eq!(table.lookup(a), Some(&1));
    }

    #[test]
    fn test_exit_root_is_a_noop() {
        let mut table: SymbolTable<()> = SymbolTable::new();
        assert!(!table.exit_scope());
        assert_eq!(table.depth(), 1);
    }

    #[test]
    fn test_define_in_inner_does_not_touch_outer() {
        let (_i, a, _b, _c) = names();
        let mut table: SymbolTable<i32> = SymbolTable::new();
        table.define(a, 1);
        table.scoped(|table| {
            table.define(a, 99);
        });
        // The inner definition vanished with its scope; the outer stands.
        assert_eq!(table.lookup(a), Some(&1));
    }

    #[test]
    fn test_unbound_lookup_is_none() {
        let (_i, a, b, _c) = names();
        let mut table: SymbolTable<i32> = SymbolTable::new();
        table.define(a, 1);
        assert_eq!(table.lookup(b), None);
        assert!(!table.is_defined(b));
    }
}
