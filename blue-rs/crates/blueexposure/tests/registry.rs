use bevy::prelude::*;
use blueexposure::prelude::*;

#[blue_pyclass(module = "BlueExposureTest", name = "TestAttributes", kind = "component")]
#[derive(Clone, Debug, Default)]
struct TestAttributes {
    #[blue(read, write, persist)]
    my_int: i32,
    #[blue(read)]
    my_name: String,
}

#[blue_methods]
impl TestAttributes {
    #[blue(new)]
    fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self) {
        self.my_int = 0;
    }
}

#[blue_test]
fn class_registration_is_bevy_native() {
    let mut app = App::new();
    app.add_plugins(BlueExposurePlugin)
        .register_blue_class::<TestAttributes>();

    let registry = app.world().resource::<BlueRegistry>();
    let info = registry
        .class("BlueExposureTest.TestAttributes")
        .expect("generated Blue class metadata must be registered");
    assert_eq!(info.name, "TestAttributes");
    assert_eq!(info.kind, BlueClassKind::Component);
    assert_eq!(info.fields.len(), 2);
    assert!(info.fields[0].readable);
    assert!(info.fields[0].writable);
    assert!(info.fields[0].persisted);
}

#[blue_test]
fn duplicate_registration_is_idempotent_by_name() {
    let mut app = App::new();
    app.add_plugins(BlueExposurePlugin)
        .register_blue_class::<TestAttributes>()
        .register_blue_class::<TestAttributes>();

    let registry = app.world().resource::<BlueRegistry>();
    assert_eq!(registry.classes().count(), 1);
}

#[blue_function(module = "BlueExposureTest", name = "AddOne", doc = "Adds one.")]
fn add_one(value: i32) -> i32 {
    value + 1
}

#[blue_enum(module = "BlueExposureTest", name = "Mode", doc = "Example exposure enum.")]
enum Mode {
    Off = 0,
    On = 1,
}

#[blue_test]
fn generated_function_descriptor_preserves_carbon_registration_metadata() {
    let info = ADD_ONE_BLUE_FUNCTION.info();
    assert_eq!(info.qualified_name(), "BlueExposureTest.AddOne");
    assert_eq!(add_one(4), 5);
}

#[blue_test]
fn generated_enum_descriptor_registers_with_bevy_registry() {
    let mut app = App::new();
    app.add_plugins(BlueExposurePlugin)
        .register_blue_enum(MODE_BLUE_ENUM);

    let registry = app.world().resource::<BlueRegistry>();
    let info = registry
        .enum_info("BlueExposureTest.Mode")
        .expect("generated enum metadata must be registered");
    assert_eq!(info.variants, vec!["Off".to_owned(), "On".to_owned()]);
}
