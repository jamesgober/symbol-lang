//! Ready-made diagnostics for the two name errors every language reports.

use alloc::format;

use diag_lang::{Diagnostic, Label, Severity, Span};

/// Builds the diagnostic for a name that is used but not bound in any scope.
///
/// The primary label points at the `reference` — where the name was used. `name`
/// is the human-readable spelling, which the caller resolves from the
/// [`Symbol`](intern_lang::Symbol) with its interner.
///
/// # Examples
///
/// ```
/// use symbol_lang::unresolved_name;
/// use diag_lang::{Severity, Span};
///
/// let diag = unresolved_name("foo", Span::new(8, 11));
/// assert_eq!(diag.severity(), Severity::Error);
/// assert_eq!(diag.message(), "cannot find `foo` in this scope");
/// assert_eq!(diag.primary().span(), Span::new(8, 11));
/// ```
#[must_use]
pub fn unresolved_name(name: &str, reference: Span) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        format!("cannot find `{name}` in this scope"),
        Label::new(reference, "not found in this scope"),
    )
}

/// Builds the diagnostic for a name defined twice in the same scope.
///
/// The primary label points at the `redefinition`, and a secondary label points
/// back at the `first` definition, so the rendered error shows both. `name` is the
/// human-readable spelling.
///
/// # Examples
///
/// ```
/// use symbol_lang::duplicate_definition;
/// use diag_lang::{Severity, Span};
///
/// let diag = duplicate_definition("x", Span::new(30, 31), Span::new(4, 5));
/// assert_eq!(diag.severity(), Severity::Error);
/// assert_eq!(diag.message(), "the name `x` is defined multiple times");
/// assert_eq!(diag.primary().span(), Span::new(30, 31));
/// assert_eq!(diag.secondary().len(), 1);
/// assert_eq!(diag.secondary()[0].span(), Span::new(4, 5));
/// ```
#[must_use]
pub fn duplicate_definition(name: &str, redefinition: Span, first: Span) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        format!("the name `{name}` is defined multiple times"),
        Label::new(redefinition, "redefined here"),
    )
    .with_secondary(Label::new(first, "first definition is here"))
}

#[cfg(test)]
mod tests {
    use diag_lang::{Severity, Span};

    use super::*;

    #[test]
    fn test_unresolved_name_points_at_the_reference() {
        let diag = unresolved_name("bar", Span::new(1, 4));
        assert_eq!(diag.severity(), Severity::Error);
        assert_eq!(diag.message(), "cannot find `bar` in this scope");
        assert_eq!(diag.primary().span(), Span::new(1, 4));
        assert!(diag.secondary().is_empty());
    }

    #[test]
    fn test_duplicate_definition_links_both_sites() {
        let diag = duplicate_definition("y", Span::new(20, 21), Span::new(2, 3));
        assert_eq!(diag.message(), "the name `y` is defined multiple times");
        assert_eq!(diag.primary().span(), Span::new(20, 21));
        assert_eq!(diag.secondary().len(), 1);
        assert_eq!(diag.secondary()[0].span(), Span::new(2, 3));
    }
}
