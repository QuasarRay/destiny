use pyo3::{prelude::*, types::PyModuleMethods};

use crate::{BlueClass, BlueEnumDescriptor, BlueFunctionDescriptor};

/// Opinionated module registrar: every class/function/enum goes through Blue metadata and a
/// single PyO3 registration convention.
pub struct BlueModule<'a, 'py> {
    module: &'a Bound<'py, PyModule>,
}

impl<'a, 'py> BlueModule<'a, 'py> {
    #[must_use]
    pub const fn new(module: &'a Bound<'py, PyModule>) -> Self {
        Self { module }
    }

    pub fn class<T>(&self) -> PyResult<&Self>
    where
        T: BlueClass,
    {
        T::register_python(self.module)?;
        Ok(self)
    }

    pub fn function(&self, descriptor: BlueFunctionDescriptor) -> PyResult<&Self> {
        descriptor.register_python(self.module)?;
        Ok(self)
    }

    pub fn enum_(&self, descriptor: BlueEnumDescriptor) -> PyResult<&Self> {
        descriptor.register_python(self.module)?;
        Ok(self)
    }

    pub fn value<V>(&self, name: &str, value: V) -> PyResult<&Self>
    where
        V: IntoPyObject<'py>,
    {
        self.module.add(name, value)?;
        Ok(self)
    }

    #[must_use]
    pub const fn module(&self) -> &'a Bound<'py, PyModule> {
        self.module
    }
}
