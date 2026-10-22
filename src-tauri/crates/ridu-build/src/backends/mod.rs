pub mod android;
pub mod linux;
pub mod web;

use crate::process::CommandSpec;
use ridu_core::models::{BuildMode, Target};
use std::path::Path;

pub trait BuildBackend: Send + Sync {
    fn target(&self) -> Target;
    fn build_command(&self, project_path: &Path, mode: BuildMode) -> CommandSpec;
}

pub use android::FlutterAndroidBackend;
pub use linux::FlutterLinuxBackend;
pub use web::FlutterWebBackend;

pub fn get_backend(target: Target) -> Box<dyn BuildBackend> {
    match target {
        Target::Web => Box::new(FlutterWebBackend),
        Target::LinuxDesktop => Box::new(FlutterLinuxBackend),
        Target::AndroidApk => Box::new(FlutterAndroidBackend::new(Target::AndroidApk)),
        Target::AndroidAab => Box::new(FlutterAndroidBackend::new(Target::AndroidAab)),
        Target::IosApp => panic!("iOS backend is orchestrated through ridu-ios engine"),
    }
}
