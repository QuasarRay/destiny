use blueexposure::{blue_methods, blue_pyclass};

#[blue_pyclass(module = "blue", name = "BlueClock", kind = "resource")]
#[derive(Clone, Debug, Default)]
pub struct BlueClock {
    #[blue(read)]
    frame: u64,
    #[blue(read)]
    ticks: u64,
}

impl BlueClock {
    pub(crate) fn advance(&mut self) {
        self.frame = self.frame.saturating_add(1);
        self.ticks = self.ticks.saturating_add(1);
    }

    #[must_use]
    pub const fn frame(&self) -> u64 {
        self.frame
    }
}

#[blue_methods]
impl BlueClock {
    #[blue(new)]
    fn py_new() -> Self {
        Self::default()
    }

    #[blue(name = "Frame")]
    fn py_frame(&self) -> u64 {
        self.frame
    }
}
