use super::BuildBackend;
use crate::builds::{BuildMode, Target};
use crate::process::CommandSpec;
use std::path::{Path, PathBuf};

pub struct FlutterWebBackend;

impl BuildBackend for FlutterWebBackend {
    fn target(&self) -> Target {
        Target::Web
    }

    fn build_command(&self, project_path: &Path, mode: BuildMode) -> CommandSpec {
        let flutter_bin = which::which("flutter").unwrap_or_else(|_| PathBuf::from("flutter"));
        let mode_arg = format!("--{}", mode);

        CommandSpec::new(flutter_bin)
            .args(["build", "web", &mode_arg])
            .current_dir(project_path)
    }
}
