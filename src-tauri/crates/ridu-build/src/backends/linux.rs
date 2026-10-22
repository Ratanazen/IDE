use super::BuildBackend;
use crate::process::CommandSpec;
use ridu_core::models::{BuildMode, Target};
use std::path::{Path, PathBuf};

pub struct FlutterLinuxBackend;

impl BuildBackend for FlutterLinuxBackend {
    fn target(&self) -> Target {
        Target::LinuxDesktop
    }

    fn build_command(&self, project_path: &Path, mode: BuildMode) -> CommandSpec {
        let flutter_bin = which::which("flutter").unwrap_or_else(|_| PathBuf::from("flutter"));
        let mode_arg = format!("--{}", mode);

        CommandSpec::new(flutter_bin)
            .args(["build", "linux", &mode_arg])
            .current_dir(project_path)
    }
}
