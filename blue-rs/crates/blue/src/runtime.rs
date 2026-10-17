use bevy::prelude::*;
use delegate::delegate;

use crate::{BlueLifecycle, BluePlugins, BlueResource, BlueResourceIndex, BlueStartupArgs};

/// Embedded Blue runtime backed directly by a Bevy `App`.
///
/// This deliberately exposes Bevy's `World` rather than hiding it behind a second object model.
pub struct BlueRuntime {
    app: App,
}

impl Default for BlueRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueRuntime {
    #[must_use]
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(BluePlugins);
        Self { app }
    }

    delegate! {
        to self.app {
            pub fn world(&self) -> &World;
            pub fn world_mut(&mut self) -> &mut World;
        }
    }

    pub fn update(&mut self) {
        let _ = self.app.update();
    }

    pub fn set_startup_args(&mut self, args: impl IntoIterator<Item = String>) {
        self.world_mut().resource_mut::<BlueStartupArgs>().replace(args);
    }

    pub fn request_shutdown(&mut self, exit_code: i32) {
        self.world_mut()
            .resource_mut::<BlueLifecycle>()
            .request_shutdown(exit_code);
    }

    pub fn terminate(&mut self) {
        self.world_mut().resource_mut::<BlueLifecycle>().terminate();
    }

    pub fn add_resource(
        &mut self,
        name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Handle<BlueResource> {
        let name = name.into();
        let handle = self
            .world_mut()
            .resource_mut::<Assets<BlueResource>>()
            .add(BlueResource::new(name.clone(), bytes));
        self.world_mut()
            .resource_mut::<BlueResourceIndex>()
            .insert(name, handle.clone());
        handle
    }

    #[must_use]
    pub fn app(&self) -> &App {
        &self.app
    }

    #[must_use]
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
}
