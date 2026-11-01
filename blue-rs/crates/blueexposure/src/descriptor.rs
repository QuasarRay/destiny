use bevy::prelude::App;
use pyo3::{prelude::*, types::PyModule};

use crate::{BlueEnumInfo, BlueFunctionInfo};

pub type BluePythonRegistrar = for<'py> fn(&Bound<'py, PyModule>) -> PyResult<()>;
pub type BlueBevyRegistrar = fn(&mut App);

#[derive(Clone, Copy)]
pub struct BlueFunctionDescriptor {
    module: &'static str,
    name: &'static str,
    doc: &'static str,
    register_python: BluePythonRegistrar,
}

impl BlueFunctionDescriptor {
    #[must_use]
    pub const fn new(
        module: &'static str,
        name: &'static str,
        doc: &'static str,
        register_python: BluePythonRegistrar,
    ) -> Self {
        Self {
            module,
            name,
            doc,
            register_python,
        }
    }

    #[must_use]
    pub fn info(self) -> BlueFunctionInfo {
        BlueFunctionInfo::new(self.module, self.name, self.doc)
    }

    pub fn register_python(self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        (self.register_python)(module)
    }
}

#[derive(Clone, Copy)]
pub struct BlueEnumDescriptor {
    module: &'static str,
    name: &'static str,
    doc: &'static str,
    variants: &'static [&'static str],
    register_python: BluePythonRegistrar,
    register_bevy: BlueBevyRegistrar,
}

impl BlueEnumDescriptor {
    #[must_use]
    pub const fn new(
        module: &'static str,
        name: &'static str,
        doc: &'static str,
        variants: &'static [&'static str],
        register_python: BluePythonRegistrar,
        register_bevy: BlueBevyRegistrar,
    ) -> Self {
        Self {
            module,
            name,
            doc,
            variants,
            register_python,
            register_bevy,
        }
    }

    #[must_use]
    pub fn info(self) -> BlueEnumInfo {
        BlueEnumInfo::new(self.module, self.name, self.doc, self.variants.iter().copied())
    }

    pub fn register_python(self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        (self.register_python)(module)
    }

    pub fn register_bevy(self, app: &mut App) {
        (self.register_bevy)(app);
    }
}
