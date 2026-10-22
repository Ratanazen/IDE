use super::BuildBackend;
use crate::builds::{BuildMode, Target};
use crate::process::CommandSpec;
use std::path::{Path, PathBuf};

pub struct FlutterAndroidBackend {
    pub target: Target,
}

impl FlutterAndroidBackend {
    pub fn new(target: Target) -> Self {
        assert!(matches!(target, Target::AndroidApk | Target::AndroidAab));
        Self { target }
    }

    /// Checks if a project contains a custom release signing key configuration (`android/key.properties`).
    pub fn has_release_signing(project_path: &Path) -> bool {
        project_path.join("android/key.properties").exists()
    }
}

impl BuildBackend for FlutterAndroidBackend {
    fn target(&self) -> Target {
        self.target
    }

    fn build_command(&self, project_path: &Path, mode: BuildMode) -> CommandSpec {
        let flutter_bin = which::which("flutter").unwrap_or_else(|_| PathBuf::from("flutter"));
        let mode_arg = format!("--{}", mode);

        let subcmd = match self.target {
            Target::AndroidApk => "apk",
            Target::AndroidAab => "appbundle",
            _ => unreachable!(),
        };

        let mut spec = CommandSpec::new(flutter_bin)
            .args(["build", subcmd, &mode_arg])
            .current_dir(project_path);

        // Prioritize JDK 17 if installed for AGP compatibility
        let jdk17 = PathBuf::from("/usr/lib/jvm/java-17-openjdk");
        if jdk17.exists() {
            spec = spec.env("JAVA_HOME", jdk17.to_string_lossy().to_string());
        }

        spec
    }
}
