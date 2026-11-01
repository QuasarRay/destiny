use blueexposure::{blue_methods, blue_pyclass};

#[blue_pyclass(module = "blue", name = "BlueObjectMetadata", kind = "component")]
#[derive(Clone, Debug, Default)]
pub struct BlueObjectMetadata {
    #[blue(read, write, persist)]
    class_name: String,
    #[blue(read, write, persist)]
    source: String,
    #[blue(read)]
    generation: u64,
}

impl BlueObjectMetadata {
    #[must_use]
    pub fn new(class_name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
            source: source.into(),
            generation: 0,
        }
    }

    pub fn touch(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }
}

#[blue_methods]
impl BlueObjectMetadata {
    #[blue(new)]
    fn py_new(class_name: String, source: Option<String>) -> Self {
        Self::new(class_name, source.unwrap_or_default())
    }

    #[blue(name = "Touch")]
    fn py_touch(&mut self) {
        self.touch();
    }
}
