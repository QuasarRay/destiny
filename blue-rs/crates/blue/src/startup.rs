use blueexposure::{blue_methods, blue_pyclass};
use delegate::delegate;

#[blue_pyclass(module = "blue", name = "BlueStartupArgs", kind = "resource")]
#[derive(Clone, Debug, Default)]
pub struct BlueStartupArgs {
    #[blue(read, write)]
    args: Vec<String>,
}

impl BlueStartupArgs {
    delegate! {
        to self.args {
            #[call(len)]
            pub fn count(&self) -> usize;
            pub fn is_empty(&self) -> bool;
            pub fn clear(&mut self);
        }
    }

    #[must_use]
    pub fn contains(&self, value: &str) -> bool {
        self.args.iter().any(|arg| arg == value)
    }

    pub fn replace(&mut self, values: impl IntoIterator<Item = String>) {
        self.args.clear();
        self.args.extend(values);
    }

    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.args
    }
}

#[blue_methods]
impl BlueStartupArgs {
    #[blue(new)]
    fn py_new(args: Option<Vec<String>>) -> Self {
        Self {
            args: args.unwrap_or_default(),
        }
    }

    #[blue(name = "Has")]
    fn py_has(&self, value: &str) -> bool {
        self.contains(value)
    }

    #[blue(name = "Set")]
    fn py_set(&mut self, args: Vec<String>) {
        self.replace(args);
    }
}
