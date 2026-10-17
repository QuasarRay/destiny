extern crate self as blueexposure;

mod app_ext;
mod callback;
mod class;
mod descriptor;
mod metadata;
mod module;
mod registry;

pub use app_ext::{BlueAppExt, BlueExposurePlugin};
pub use blue_macros::{blue_enum, blue_function, blue_methods, blue_pyclass, blue_test, BlueClass};
pub use callback::BlueScriptCallback;
pub use class::BlueClass;
pub use descriptor::{BlueEnumDescriptor, BlueFunctionDescriptor};
pub use metadata::{
    BlueClassInfo, BlueClassKind, BlueEnumInfo, BlueFieldInfo, BlueFunctionInfo, BlueLegacyTestInfo,
};
pub use module::BlueModule;
pub use registry::BlueRegistry;

pub mod prelude {
    pub use crate::{
        blue_enum, blue_function, blue_methods, blue_pyclass, blue_test, BlueAppExt, BlueClass,
        BlueClassInfo, BlueClassKind, BlueEnumDescriptor, BlueEnumInfo, BlueExposurePlugin,
        BlueFieldInfo, BlueFunctionDescriptor, BlueFunctionInfo, BlueModule, BlueRegistry,
        BlueScriptCallback,
    };
}
