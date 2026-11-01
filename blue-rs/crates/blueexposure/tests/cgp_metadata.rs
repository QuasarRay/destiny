use bevy::prelude::*;
use blueexposure::{
    BlueClassInfo, BlueClassKind, BlueFieldInfo, BlueMetadataShape,
    BlueMetadataWriter, BlueMetadataWriterComponent,
    BlueRegistration, BlueRegistry, CanWriteBlueMetadata,
};
use cgp::prelude::*;

#[test]
fn derive_deftly_schema_is_generated_from_metadata_fields() {
    assert_eq!(
        BlueClassInfo::rust_field_names(),
        &["module", "name", "doc", "kind", "fields"]
    );
    assert_eq!(
        BlueFieldInfo::rust_field_names(),
        &["name", "doc", "readable", "writable", "persisted"]
    );
}

#[test]
fn cgp_default_provider_preserves_registration_upsert_semantics() {
    let mut app = App::new();
    app.insert_resource(BlueRegistry::default());
    let mut reg = app.world_mut().resource_mut::<BlueRegistry>();
    reg.register_class(BlueClassInfo::new("test", "Item", "first", BlueClassKind::Component, vec![]));
    reg.register_class(BlueClassInfo::new("test", "Item", "second", BlueClassKind::Resource, vec![]));
    assert_eq!(reg.class_count(), 1);
    assert_eq!(reg.class("test.Item").unwrap().doc, "second");
}

/// A genuinely different writer context/provider wired at compile time.
#[derive(Default)]
struct CountingRegistrations(usize);

struct CountOnly;

#[cgp_provider]
impl BlueMetadataWriter<CountingRegistrations> for CountOnly {
    fn write_blue_metadata(ctx: &mut CountingRegistrations, _metadata: BlueRegistration) {
        ctx.0 += 1;
    }
}

delegate_components! {
    CountingRegistrations {
        BlueMetadataWriterComponent: CountOnly,
    }
}

#[test]
fn cgp_provider_is_substitutable_without_changing_consumer() {
    let mut context = CountingRegistrations::default();
    context.write_blue_metadata(BlueRegistration::Class(BlueClassInfo::new(
        "test", "Item", "", BlueClassKind::Component, vec![],
    )));
    assert_eq!(context.0, 1);
}
