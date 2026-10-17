use bevy::reflect::GetTypeRegistration;
use pyo3::{prelude::*, types::PyModuleMethods, PyClass};

use crate::BlueClassInfo;

/// Contract generated for every Blue-exposed Rust class.
///
/// The class remains a normal Bevy reflected type and a normal PyO3 class; this trait only
/// centralizes Carbon/Blue registration metadata so `App` and Python module registration use
/// the same source of truth.
pub trait BlueClass: PyClass + GetTypeRegistration + Send + Sync + 'static {
    const BLUE_MODULE: &'static str;
    const BLUE_NAME: &'static str;

    fn blue_class_info() -> BlueClassInfo;

    fn register_python(module: &Bound<'_, PyModule>) -> PyResult<()>
    where
        Self: Sized,
    {
        module.add_class::<Self>()
    }
}
