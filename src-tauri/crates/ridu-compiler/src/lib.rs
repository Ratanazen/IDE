use ridu_core::models::{BuildCommand, CapabilityStatus, ToolchainInfo};
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Tool '{0}' is not installed or available on this system")]
    ToolNotFound(String),

    #[error("Missing SDK sysroot for target '{0}'")]
    MissingSysroot(String),

    #[error("Compiler error: {0}")]
    ExecutionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub struct CompilerDiscovery;

impl CompilerDiscovery {
    pub fn discover_tools() -> Vec<ToolchainInfo> {
        vec![
            Self::probe_clang(),
            Self::probe_clang_cpp(),
            Self::probe_ld64(),
            Self::probe_swiftc(),
        ]
    }

    pub fn probe_clang() -> ToolchainInfo {
        let path = which::which("clang").ok();
        let (version, status) = if let Some(ref p) = path {
            let ver = std::process::Command::new(p)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.lines().next().map(|l| l.trim().to_string())
                });
            (ver, CapabilityStatus::Verified)
        } else {
            (None, CapabilityStatus::MissingDependency)
        };

        ToolchainInfo {
            id: "clang".to_string(),
            name: "LLVM Clang (C / Objective-C)".to_string(),
            binary_path: path,
            version,
            status,
            triple: Some("arm64-apple-ios".to_string()),
            details: Some("Direct native Darwin cross-compilation target support".to_string()),
            documentation_url: Some("https://clang.llvm.org/".to_string()),
        }
    }

    pub fn probe_clang_cpp() -> ToolchainInfo {
        let path = which::which("clang++").ok();
        let (version, status) = if let Some(ref p) = path {
            let ver = std::process::Command::new(p)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.lines().next().map(|l| l.trim().to_string())
                });
            (ver, CapabilityStatus::Verified)
        } else {
            (None, CapabilityStatus::MissingDependency)
        };

        ToolchainInfo {
            id: "clang++".to_string(),
            name: "LLVM Clang++ (C++ / Objective-C++)".to_string(),
            binary_path: path,
            version,
            status,
            triple: Some("arm64-apple-ios".to_string()),
            details: Some("C++ and Objective-C++ compiler driver".to_string()),
            documentation_url: Some("https://clang.llvm.org/".to_string()),
        }
    }

    pub fn probe_ld64() -> ToolchainInfo {
        let path = which::which("ld64.lld").ok();
        let (version, status) = if let Some(ref p) = path {
            let ver = std::process::Command::new(p)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.lines().next().map(|l| l.trim().to_string())
                });
            (ver, CapabilityStatus::Verified)
        } else {
            (None, CapabilityStatus::MissingDependency)
        };

        ToolchainInfo {
            id: "ld64.lld".to_string(),
            name: "LLVM Mach-O Linker (ld64.lld)".to_string(),
            binary_path: path,
            version,
            status,
            triple: Some("arm64-apple-darwin".to_string()),
            details: Some("Links Mach-O 64-bit object files and Apple framework dependencies".to_string()),
            documentation_url: Some("https://lld.llvm.org/".to_string()),
        }
    }

    pub fn probe_swiftc() -> ToolchainInfo {
        let path = which::which("swiftc").ok().or_else(|| which::which("swift").ok());
        let (version, status) = if let Some(ref p) = path {
            let ver = std::process::Command::new(p)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.lines().next().map(|l| l.trim().to_string())
                });
            (ver, CapabilityStatus::Verified)
        } else {
            (None, CapabilityStatus::MissingDependency)
        };

        ToolchainInfo {
            id: "swiftc".to_string(),
            name: "Swift Compiler (swiftc)".to_string(),
            binary_path: path,
            version,
            status,
            triple: Some("arm64-apple-ios".to_string()),
            details: Some("Available on Linux via chaotic-aur/swift-bin or swift.org".to_string()),
            documentation_url: Some("https://swift.org/".to_string()),
        }
    }
}

/// Invocation builder for LLVM Clang (C, C++, Objective-C, Objective-C++)
pub struct ClangInvocationBuilder {
    target_triple: String,
    sysroot: Option<PathBuf>,
    include_paths: Vec<PathBuf>,
    framework_paths: Vec<PathBuf>,
    defines: Vec<String>,
    flags: Vec<String>,
    is_arc_enabled: bool,
    is_modules_enabled: bool,
    is_cpp: bool,
}

impl ClangInvocationBuilder {
    pub fn new(target_triple: &str) -> Self {
        Self {
            target_triple: target_triple.to_string(),
            sysroot: None,
            include_paths: Vec::new(),
            framework_paths: Vec::new(),
            defines: Vec::new(),
            flags: Vec::new(),
            is_arc_enabled: true,
            is_modules_enabled: true,
            is_cpp: false,
        }
    }

    pub fn sysroot<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.sysroot = Some(path.into());
        self
    }

    pub fn add_include<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.include_paths.push(path.into());
        self
    }

    pub fn add_framework_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.framework_paths.push(path.into());
        self
    }

    pub fn add_define<S: Into<String>>(mut self, define: S) -> Self {
        self.defines.push(define.into());
        self
    }

    pub fn cpp(mut self, is_cpp: bool) -> Self {
        self.is_cpp = is_cpp;
        self
    }

    pub fn build_command(&self, source_file: &Path, output_object: &Path) -> BuildCommand {
        let binary_name = if self.is_cpp { "clang++" } else { "clang" };
        let mut cmd = BuildCommand::new(binary_name);

        cmd = cmd.arg("-target").arg(&self.target_triple);

        if let Some(ref sysroot) = self.sysroot {
            cmd = cmd.arg("-isysroot").arg(sysroot.to_string_lossy().to_string());
        }

        for inc in &self.include_paths {
            cmd = cmd.arg("-I").arg(inc.to_string_lossy().to_string());
        }

        for fw in &self.framework_paths {
            cmd = cmd.arg("-F").arg(fw.to_string_lossy().to_string());
        }

        for def in &self.defines {
            cmd = cmd.arg(format!("-D{}", def));
        }

        if self.is_arc_enabled {
            cmd = cmd.arg("-fobjc-arc");
        }

        if self.is_modules_enabled {
            cmd = cmd.arg("-fmodules");
        }

        for flag in &self.flags {
            cmd = cmd.arg(flag);
        }

        cmd = cmd
            .arg("-c")
            .arg(source_file.to_string_lossy().to_string())
            .arg("-o")
            .arg(output_object.to_string_lossy().to_string());

        cmd
    }
}

/// Invocation builder for Apple Mach-O Linker (ld64.lld)
pub struct Ld64InvocationBuilder {
    arch: String,
    platform: String,
    min_version: String,
    sdk_version: String,
    syslibroot: Option<PathBuf>,
    framework_paths: Vec<PathBuf>,
    frameworks: Vec<String>,
    library_paths: Vec<PathBuf>,
    libraries: Vec<String>,
    rpaths: Vec<String>,
}

impl Ld64InvocationBuilder {
    pub fn new(arch: &str, min_ios_version: &str) -> Self {
        Self {
            arch: arch.to_string(),
            platform: "ios".to_string(),
            min_version: min_ios_version.to_string(),
            sdk_version: min_ios_version.to_string(),
            syslibroot: None,
            framework_paths: Vec::new(),
            frameworks: vec!["Foundation".to_string()],
            library_paths: Vec::new(),
            libraries: vec!["c++".to_string()],
            rpaths: vec!["@executable_path/Frameworks".to_string()],
        }
    }

    pub fn syslibroot<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.syslibroot = Some(path.into());
        self
    }

    pub fn add_framework<S: Into<String>>(mut self, fw: S) -> Self {
        self.frameworks.push(fw.into());
        self
    }

    pub fn add_framework_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.framework_paths.push(path.into());
        self
    }

    pub fn add_library_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.library_paths.push(path.into());
        self
    }

    pub fn add_rpath<S: Into<String>>(mut self, rpath: S) -> Self {
        self.rpaths.push(rpath.into());
        self
    }

    pub fn build_command(&self, object_files: &[PathBuf], output_executable: &Path) -> BuildCommand {
        let binary = which::which("ld64.lld")
            .unwrap_or_else(|_| PathBuf::from("ld64.lld"));
        let mut cmd = BuildCommand::new(binary);

        cmd = cmd.arg("-arch").arg(&self.arch);
        cmd = cmd.arg("-platform_version")
            .arg(&self.platform)
            .arg(&self.min_version)
            .arg(&self.sdk_version);

        if let Some(ref root) = self.syslibroot {
            cmd = cmd.arg("-syslibroot").arg(root.to_string_lossy().to_string());
        }

        for fw_path in &self.framework_paths {
            cmd = cmd.arg("-F").arg(fw_path.to_string_lossy().to_string());
        }

        for lib_path in &self.library_paths {
            cmd = cmd.arg("-L").arg(lib_path.to_string_lossy().to_string());
        }

        for fw in &self.frameworks {
            cmd = cmd.arg("-framework").arg(fw);
        }

        for lib in &self.libraries {
            cmd = cmd.arg(format!("-l{}", lib));
        }


        for rpath in &self.rpaths {
            cmd = cmd.arg("-rpath").arg(rpath);
        }

        for obj in object_files {
            cmd = cmd.arg(obj.to_string_lossy().to_string());
        }

        cmd = cmd.arg("-o").arg(output_executable.to_string_lossy().to_string());
        cmd
    }
}

/// Invocation builder for Swift Compiler (swiftc)
pub struct SwiftInvocationBuilder {
    target_triple: String,
    sdk_path: Option<PathBuf>,
    module_name: String,
    import_paths: Vec<PathBuf>,
    framework_paths: Vec<PathBuf>,
    bridging_header: Option<PathBuf>,
}

impl SwiftInvocationBuilder {
    pub fn new(module_name: &str, target_triple: &str) -> Self {
        Self {
            target_triple: target_triple.to_string(),
            sdk_path: None,
            module_name: module_name.to_string(),
            import_paths: Vec::new(),
            framework_paths: Vec::new(),
            bridging_header: None,
        }
    }

    pub fn sdk<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.sdk_path = Some(path.into());
        self
    }

    pub fn bridging_header<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.bridging_header = Some(path.into());
        self
    }

    pub fn build_command(&self, sources: &[PathBuf], output_module_dir: &Path) -> BuildCommand {
        let binary = which::which("swiftc")
            .unwrap_or_else(|_| PathBuf::from("swiftc"));
        let mut cmd = BuildCommand::new(binary);

        cmd = cmd.arg("-target").arg(&self.target_triple);
        cmd = cmd.arg("-module-name").arg(&self.module_name);

        if let Some(ref sdk) = self.sdk_path {
            cmd = cmd.arg("-sdk").arg(sdk.to_string_lossy().to_string());
        }

        if let Some(ref header) = self.bridging_header {
            cmd = cmd.arg("-import-objc-header").arg(header.to_string_lossy().to_string());
        }

        for inc in &self.import_paths {
            cmd = cmd.arg("-I").arg(inc.to_string_lossy().to_string());
        }

        for fw in &self.framework_paths {
            cmd = cmd.arg("-F").arg(fw.to_string_lossy().to_string());
        }

        cmd = cmd.arg("-emit-module");
        cmd = cmd.arg("-emit-module-path").arg(
            output_module_dir
                .join(format!("{}.swiftmodule", self.module_name))
                .to_string_lossy()
                .to_string(),
        );

        for src in sources {
            cmd = cmd.arg(src.to_string_lossy().to_string());
        }

        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clang_invocation_construction() {
        let builder = ClangInvocationBuilder::new("arm64-apple-ios16.0")
            .sysroot("/opt/iOS-SDKs/iPhoneOS16.0.sdk")
            .add_include("include")
            .add_framework_path("Frameworks")
            .add_define("DEBUG=1");

        let cmd = builder.build_command(Path::new("main.m"), Path::new("build/main.o"));
        assert_eq!(cmd.program.to_string_lossy(), "clang");
        assert!(cmd.args.contains(&"-target".to_string()));
        assert!(cmd.args.contains(&"arm64-apple-ios16.0".to_string()));
        assert!(cmd.args.contains(&"-isysroot".to_string()));
        assert!(cmd.args.contains(&"/opt/iOS-SDKs/iPhoneOS16.0.sdk".to_string()));
        assert!(cmd.args.contains(&"-fobjc-arc".to_string()));
        assert!(cmd.args.contains(&"-DDEBUG=1".to_string()));
    }

    #[test]
    fn test_ld64_lld_invocation_construction() {
        let builder = Ld64InvocationBuilder::new("arm64", "16.0")
            .syslibroot("/opt/iOS-SDKs/iPhoneOS16.0.sdk")
            .add_framework("UIKit")
            .add_rpath("@executable_path/Frameworks");

        let cmd = builder.build_command(
            &[PathBuf::from("build/main.o"), PathBuf::from("build/AppDelegate.o")],
            Path::new("build/Runner.app/Runner"),
        );

        assert!(cmd.program.to_string_lossy().contains("ld64.lld"));
        assert!(cmd.args.contains(&"-arch".to_string()));
        assert!(cmd.args.contains(&"arm64".to_string()));
        assert!(cmd.args.contains(&"-platform_version".to_string()));
        assert!(cmd.args.contains(&"UIKit".to_string()));
        assert!(cmd.args.contains(&"Foundation".to_string()));
        assert!(cmd.args.contains(&"-rpath".to_string()));
        assert!(cmd.args.contains(&"@executable_path/Frameworks".to_string()));
    }

    #[test]
    fn test_compiler_discovery_probes() {
        let tools = CompilerDiscovery::discover_tools();
        assert_eq!(tools.len(), 4);

        let clang = tools.iter().find(|t| t.id == "clang").unwrap();
        assert_eq!(clang.status, CapabilityStatus::Verified);

        let ld64 = tools.iter().find(|t| t.id == "ld64.lld").unwrap();
        assert_eq!(ld64.status, CapabilityStatus::Verified);
    }
}
