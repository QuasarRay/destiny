//! Total machine-counter updates shared by the actual ABI and Verus.
//!
//! The declaration macro is the only adapter: normal Rust emits ordinary
//! functions; Verus emits the identical bodies with checked contracts. Neither
//! adapter supplies a replacement operation or changes integer arithmetic.

macro_rules! ffi_lifecycle_bodies {
    ($declare:ident) => {
        $declare! {
            retain_count(count: usize) -> Option<usize> {
                #[cfg(kani)]
                kani::cover!(true,"actual production retain counter body executed");
                if count == 0 || count == usize::MAX {
                    None
                } else {
                    Some(count + 1)
                }
            }
        }
        $declare! {
            release_count(count: usize) -> Option<usize> {
                #[cfg(kani)]
                kani::cover!(true,"actual production release counter body executed");
                if count == 0 {
                    None
                } else {
                    Some(count - 1)
                }
            }
        }
    };
}
#[cfg(verus_keep_ghost)]
pub(crate) use ffi_lifecycle_bodies;

#[cfg(not(verus_keep_ghost))]
macro_rules! rust_lifecycle_function {
    ($name:ident($count:ident: usize) -> Option<usize> $body:block) => {
        pub(crate) fn $name($count: usize) -> Option<usize> $body
    };
}

#[cfg(not(verus_keep_ghost))]
ffi_lifecycle_bodies!(rust_lifecycle_function);

// One atomic retry attempt has the same control body in production and Verus.
// The CAS adapter differs only in the trusted atomic primitive/ghost permission.
macro_rules! ffi_reference_attempt_bodies {
    ($declare:ident,$cas:ident) => {
        $declare! {
            retain_attempt(counter,expected,permission) -> Option<Result<usize,usize>> {
                match retain_count(expected) {
                    None => None,
                    Some(next) => Some($cas!(counter,permission,expected,next)),
                }
            }
        }
        $declare! {
            release_attempt(counter,expected,permission) -> Option<Result<usize,usize>> {
                match release_count(expected) {
                    None => None,
                    Some(next) => Some($cas!(counter,permission,expected,next)),
                }
            }
        }
    };
}
#[cfg(verus_keep_ghost)]
pub(crate) use ffi_reference_attempt_bodies;

#[cfg(not(verus_keep_ghost))]
macro_rules! rust_reference_attempt {
    ($name:ident($counter:ident,$expected:ident,$permission:ident) -> Option<Result<usize,usize>> $body:block) => {
        pub(crate) fn $name($counter:&std::sync::atomic::AtomicUsize,$expected:usize)->Option<Result<usize,usize>> $body
    };
}
#[cfg(not(verus_keep_ghost))]
macro_rules! rust_reference_cas {
    ($counter:ident,$permission:ident,$expected:ident,$next:ident) => {
        $counter.compare_exchange_weak(
            $expected,
            $next,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
    };
}
#[cfg(not(verus_keep_ghost))]
ffi_reference_attempt_bodies!(rust_reference_attempt, rust_reference_cas);
