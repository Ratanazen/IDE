pub mod linux;
pub mod web;

use crate::builds::{BuildMode, Target};
use crate::process::CommandSpec;
use std::path::Path;

pub trait BuildBackend: Send + Sync {
    fn target(&self) -> Target;
    fn build_command(&self, project_path: &Path, mode: BuildMode) -> CommandSpec;
}

pub use linux::FlutterLinuxBackend;
pub use web::FlutterWebBackend;

pub fn get_backend(target: Target) -> Box<dyn BuildBackend> {
    match target {
        Target::Web => Box::new(FlutterWebBackend),
        Target::Linux => Box::new(FlutterLinuxBackend),
        Target::AndroidApk => unimplemented!("Android APK backend implemented in Milestone 4"),
        Target::AndroidAab => unimplemented!("Android AAB backend implemented in Milestone 4"),
        Target::Ios => unimplemented!("iOS backend implemented in Milestone 9"),
    }
}
