use blue::{
    BlueClock, BlueLifecycle, BlueLifecycleState, BlueObjectMetadata, BluePaths, BlueRuntime,
    BlueStartupArgs,
};
use blueexposure::{BlueModule, BlueScriptCallback};
use pyo3::{prelude::*, types::PyModuleMethods};

#[pyclass(name = "Runtime", module = "_bluepycore", unsendable)]
pub struct PyBlueRuntime {
    runtime: BlueRuntime,
}

#[pymethods]
impl PyBlueRuntime {
    #[new]
    fn new() -> Self {
        Self {
            runtime: BlueRuntime::new(),
        }
    }

    #[pyo3(name = "Update")]
    fn update(&mut self) {
        self.runtime.update();
    }

    #[pyo3(name = "SetStartupArgs")]
    fn set_startup_args(&mut self, args: Vec<String>) {
        self.runtime.set_startup_args(args);
    }

    #[pyo3(name = "HasStartupArg")]
    fn has_startup_arg(&self, name: &str) -> bool {
        self.runtime
            .world()
            .resource::<BlueStartupArgs>()
            .contains(name)
    }

    #[pyo3(name = "Terminate")]
    fn terminate(&mut self, exit_code: i32) {
        self.runtime.request_shutdown(exit_code);
        self.runtime.terminate();
    }

    #[getter]
    fn frame(&self) -> u64 {
        self.runtime.world().resource::<BlueClock>().frame()
    }

    #[getter]
    fn lifecycle(&self) -> String {
        self.runtime.world().resource::<BlueLifecycle>().state().to_string()
    }

    #[pyo3(name = "IsRunning")]
    fn is_running(&self) -> bool {
        self.runtime.world().resource::<BlueLifecycle>().state() == BlueLifecycleState::Running
    }
}

#[pyfunction(name = "ModuleStartup")]
fn module_startup() -> PyBlueRuntime {
    PyBlueRuntime::new()
}

#[pymodule]
fn _bluepycore(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.gil_used(true)?;
    let blue = BlueModule::new(module);
    blue.class::<BlueStartupArgs>()?
        .class::<BluePaths>()?
        .class::<BlueClock>()?
        .class::<BlueObjectMetadata>()?
        .class::<BlueScriptCallback>()?;

    module.add_class::<PyBlueRuntime>()?;
    module.add_function(wrap_pyfunction!(module_startup, module)?)?;
    Ok(())
}
