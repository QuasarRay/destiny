use pyo3::{prelude::*, types::PyAny};

use crate::{blue_methods, blue_pyclass};

#[blue_pyclass(module = "blueexposure", name = "BlueScriptCallback", kind = "component")]
#[derive(Default)]
pub struct BlueScriptCallback {
    #[blue(skip)]
    #[reflect(ignore)]
    callback: Option<Py<PyAny>>,
}

#[blue_methods]
impl BlueScriptCallback {
    #[blue(new)]
    fn py_new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.callback = None;
    }

    pub fn set(&mut self, callback: Py<PyAny>) {
        self.callback = Some(callback);
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.callback.is_some()
    }

    pub fn get(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.callback.as_ref().map(|callback| callback.clone_ref(py))
    }

    pub fn call0(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.callback
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Blue callback is empty"))?
            .call0(py)
    }

    pub fn call1(&self, py: Python<'_>, arg: Py<PyAny>) -> PyResult<Py<PyAny>> {
        self.callback
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Blue callback is empty"))?
            .call1(py, (arg,))
    }
}
