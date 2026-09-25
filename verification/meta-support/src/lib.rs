#![forbid(unsafe_code)]

//! Declarative and delegation macros shared by Kani and ordinary regression
//! tests.  Verus has its own syntax-facing wrappers but consumes the same
//! source-backed contract names.

#[macro_export]
macro_rules! regression_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() $body
    };
}

#[macro_export]
macro_rules! kani_regression {
    ($name:ident, $body:block) => {
        #[cfg(kani)]
        #[kani::proof]
        fn $name() $body
    };
}

/// Generate a thin representation delegate instead of hand-writing repeated
/// wrappers.  The target remains explicit and reviewable.
#[macro_export]
macro_rules! delegate_unary {
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($arg:ident : $arg_ty:ty) -> $ret:ty => $target:path
    ) => {
        $(#[$meta])*
        $vis fn $name($arg: $arg_ty) -> $ret {
            $target($arg)
        }
    };
}

/// Shared pattern for original Destiny setters where negative values are
/// specified as no-ops.
#[macro_export]
macro_rules! non_negative_setter_contract {
    ($name:ident, $initial:expr, $request:expr) => {
        let initial = $initial;
        let request = $request;
        let observed = destiny_original_spec::apply_non_negative_setter(initial, request);
        if request < 0.0 {
            assert!(observed.to_bits() == initial.to_bits());
        } else {
            assert!(observed.to_bits() == request.to_bits());
        }
    };
}

/// Generate both a conventional regression and a Kani harness from one body.
/// The body must be deterministic for the conventional test; symbolic Kani
/// cases should use `kani_regression!` directly.
#[macro_export]
macro_rules! paired_regression {
    ($test_name:ident, $kani_name:ident, $body:block) => {
        $crate::regression_test!($test_name, $body);
        $crate::kani_regression!($kani_name, $body);
    };
}
