use bevy::prelude::{App, Plugin};

use crate::{
    BlueClass, BlueClassInfo, BlueClassKind, BlueEnumDescriptor, BlueEnumInfo, BlueFieldInfo,
    BlueFunctionDescriptor, BlueFunctionInfo, BlueLegacyTestInfo, BlueRegistry,
};

#[derive(Default)]
pub struct BlueExposurePlugin;

impl Plugin for BlueExposurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueRegistry>()
            .register_type::<BlueRegistry>()
            .register_type::<BlueClassKind>()
            .register_type::<BlueFieldInfo>()
            .register_type::<BlueClassInfo>()
            .register_type::<BlueFunctionInfo>()
            .register_type::<BlueEnumInfo>()
            .register_type::<BlueLegacyTestInfo>();
    }
}

pub trait BlueAppExt {
    fn register_blue_class<T>(&mut self) -> &mut Self
    where
        T: BlueClass;

    fn register_blue_function(&mut self, descriptor: BlueFunctionDescriptor) -> &mut Self;

    fn register_blue_enum(&mut self, descriptor: BlueEnumDescriptor) -> &mut Self;
}

impl BlueAppExt for App {
    fn register_blue_class<T>(&mut self) -> &mut Self
    where
        T: BlueClass,
    {
        self.init_resource::<BlueRegistry>();
        self.register_type::<T>();
        self.world_mut()
            .resource_mut::<BlueRegistry>()
            .register_class(T::blue_class_info());
        self
    }

    fn register_blue_function(&mut self, descriptor: BlueFunctionDescriptor) -> &mut Self {
        self.init_resource::<BlueRegistry>();
        self.world_mut()
            .resource_mut::<BlueRegistry>()
            .register_function(descriptor.info());
        self
    }

    fn register_blue_enum(&mut self, descriptor: BlueEnumDescriptor) -> &mut Self {
        self.init_resource::<BlueRegistry>();
        descriptor.register_bevy(self);
        self.world_mut()
            .resource_mut::<BlueRegistry>()
            .register_enum(descriptor.info());
        self
    }
}
