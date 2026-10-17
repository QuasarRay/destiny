use bevy::prelude::{Reflect, Resource};
use derive_more::Display;

#[derive(Clone, Copy, Debug, Default, Display, Eq, PartialEq, Reflect)]
pub enum BlueLifecycleState {
    #[default]
    Starting,
    Running,
    ShuttingDown,
    Terminated,
}

#[derive(Clone, Debug, Reflect, Resource)]
#[reflect(Resource)]
pub struct BlueLifecycle {
    state: BlueLifecycleState,
    exit_code: i32,
}

impl Default for BlueLifecycle {
    fn default() -> Self {
        Self {
            state: BlueLifecycleState::Starting,
            exit_code: 0,
        }
    }
}

impl BlueLifecycle {
    #[must_use]
    pub const fn state(&self) -> BlueLifecycleState {
        self.state
    }

    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn mark_running(&mut self) {
        if self.state == BlueLifecycleState::Starting {
            self.state = BlueLifecycleState::Running;
        }
    }

    pub fn request_shutdown(&mut self, exit_code: i32) {
        self.exit_code = exit_code;
        self.state = BlueLifecycleState::ShuttingDown;
    }

    pub fn terminate(&mut self) {
        self.state = BlueLifecycleState::Terminated;
    }
}
