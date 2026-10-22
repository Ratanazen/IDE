use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetPlatform {
    Web,
    LinuxDesktop,
    AndroidApk,
    AndroidAab,
    IosApp,
}

pub type Target = TargetPlatform;

impl std::fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetPlatform::Web => write!(f, "web"),
            TargetPlatform::LinuxDesktop => write!(f, "linux"),
            TargetPlatform::AndroidApk => write!(f, "apk"),
            TargetPlatform::AndroidAab => write!(f, "appbundle"),
            TargetPlatform::IosApp => write!(f, "ios"),
        }
    }
}

// For backwards-compatibility with code using Target::Linux or Target::Ios
#[allow(non_upper_case_globals)]
impl TargetPlatform {
    pub const Linux: TargetPlatform = TargetPlatform::LinuxDesktop;
    pub const Ios: TargetPlatform = TargetPlatform::IosApp;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildMode {
    Debug,
    Profile,
    Release,
}

impl std::fmt::Display for BuildMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildMode::Debug => write!(f, "debug"),
            BuildMode::Profile => write!(f, "profile"),
            BuildMode::Release => write!(f, "release"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildPhase {
    Idle,
    Validating,
    ResolvingDependencies,
    Building,
    VerifyingArtifact,
    Succeeded,
    Failed,
    Cancelled,
}

impl BuildPhase {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    pub fn can_transition_to(&self, next: BuildPhase) -> bool {
        match (self, next) {
            (s, BuildPhase::Cancelled) if !s.is_terminal() => true,
            (s, BuildPhase::Failed) if !s.is_terminal() => true,
            (BuildPhase::Idle, BuildPhase::Validating) => true,
            (BuildPhase::Validating, BuildPhase::ResolvingDependencies) => true,
            (BuildPhase::Validating, BuildPhase::Building) => true,
            (BuildPhase::ResolvingDependencies, BuildPhase::Building) => true,
            (BuildPhase::Building, BuildPhase::VerifyingArtifact) => true,
            (BuildPhase::VerifyingArtifact, BuildPhase::Succeeded) => true,
            (s, BuildPhase::Idle) if s.is_terminal() => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    Verified,
    Experimental,
    Unsupported,
    Blocked,
    NotTested,
    MissingDependency,
}

pub type IosCapabilityStatus = CapabilityStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpaValidationStatus {
    PackageCreated,
    PackageValidated,
    SigningVerified,
    InstallTestPassed,
    DistributionReady,
    PackageCreatedButNotFullyVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JdkInfo {
    pub version: String,
    pub path: PathBuf,
    pub is_agp_compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidSdkInfo {
    pub path: PathBuf,
    pub platforms: Vec<String>,
    pub build_tools: Vec<String>,
    pub has_cmdline_tools: bool,
    pub licenses_accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosSdkInfo {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub platform: String,
    pub is_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftToolchainInfo {
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub available: bool,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClangToolchainInfo {
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub available: bool,
    pub supports_darwin_target: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainCapabilities {
    pub host_os: String,
    pub web: CapabilityStatus,
    pub linux_desktop: CapabilityStatus,
    pub android_apk: CapabilityStatus,
    pub android_aab: CapabilityStatus,
    pub ios_app: CapabilityStatus,
    pub ios_simulator: CapabilityStatus,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainReport {
    pub flutter: ToolStatus,
    pub dart: ToolStatus,
    pub android: Option<AndroidSdkInfo>,
    pub jdks: Vec<JdkInfo>,
    pub linux_deps: HashMap<String, ToolStatus>,
    pub ready_for_web: bool,
    pub ready_for_linux: bool,
    pub ready_for_android: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStage {
    pub name: String,
    pub phase: BuildPhase,
    pub description: String,
    pub status: StageStatus,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStageResult {
    pub stage_name: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequest {
    pub project_path: PathBuf,
    pub target: TargetPlatform,
    pub mode: BuildMode,
    pub clean_before_build: bool,
    pub run_analyze: bool,
    pub run_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPlan {
    pub id: String,
    pub project_name: String,
    pub target: TargetPlatform,
    pub mode: BuildMode,
    pub stages: Vec<BuildStage>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildReport {
    pub id: String,
    pub project_name: String,
    pub target: TargetPlatform,
    pub mode: BuildMode,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub success: bool,
    pub stages: Vec<BuildStageResult>,
    pub artifacts: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub job_id: String,
    pub project_path: PathBuf,
    pub target: TargetPlatform,
    pub mode: BuildMode,
    pub status: BuildPhase,
    pub exit_code: Option<i32>,
    pub artifact_paths: Vec<PathBuf>,
    pub log_path: Option<PathBuf>,
    pub error_code: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessOutputEvent {
    Stdout(String),
    Stderr(String),
    Finished { exit_code: Option<i32> },
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub path: PathBuf,
    pub platforms: Vec<String>,
    pub has_android: bool,
    pub has_ios: bool,
    pub has_linux: bool,
    pub has_web: bool,
    pub flutter_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub project_name: String,
    pub organization: Option<String>,
    pub parent_directory: PathBuf,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectActionResult {
    pub action: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub target_platform_id: String,
    pub is_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosProjectInfo {
    pub project_path: PathBuf,
    pub has_ios_dir: bool,
    pub bundle_id: Option<String>,
    pub app_name: Option<String>,
    pub display_name: Option<String>,
    pub deployment_target: Option<String>,
    pub uses_swift: bool,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCompatibilityReport {
    pub plugin_name: String,
    pub supports_ios: bool,
    pub has_native_code: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpaValidationReport {
    pub status: IpaValidationStatus,
    pub app_name: Option<String>,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub has_payload_dir: bool,
    pub has_app_bundle: bool,
    pub has_code_signature: bool,
    pub has_info_plist: bool,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    pub bundle_id: String,
    pub bundle_name: String,
    pub executable_name: String,
    pub version: String,
    pub short_version: String,
    pub minimum_os_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationResult {
    pub is_signed: bool,
    pub is_ad_hoc: bool,
    pub authority: Option<String>,
    pub team_id: Option<String>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosToolchainReport {
    pub host_os: String,
    pub flutter_version: Option<String>,
    pub swift_version: Option<String>,
    pub clang_version: Option<String>,
    pub can_compile: CapabilityStatus,
    pub can_package_ipa: CapabilityStatus,
    pub can_sign: CapabilityStatus,
    pub can_install_to_device: CapabilityStatus,
    pub limitations: Vec<String>,
}

/// Helper struct for constructing external commands with environment and directory overrides.
#[derive(Debug, Clone)]
pub struct BuildCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
}

impl BuildCommand {
    pub fn new<P: Into<PathBuf>>(program: P) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: HashMap::new(),
        }
    }

    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    pub fn cwd<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    pub fn env<K: Into<String>, V: Into<String>>(mut self, key: K, val: V) -> Self {
        self.env.insert(key.into(), val.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Ecosystem Public API Structures (VS Code + Xcode Hybrid Architecture)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectKind {
    Flutter,
    SwiftPackage,
    XcodeProject,
    ClangNative,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub kind: ProjectKind,
    pub root_directory: PathBuf,
    pub active_scheme: Option<String>,
    pub active_configuration: Option<String>,
    pub active_target: Option<String>,
    pub targets: Vec<BuildTarget>,
    pub schemes: Vec<Scheme>,
    pub configurations: Vec<BuildConfiguration>,
    pub deployment_target: Option<String>,
    pub bundle_identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductType {
    Application,
    Framework,
    StaticLibrary,
    DynamicLibrary,
    CommandLineTool,
    UnitTestBundle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    pub id: String,
    pub name: String,
    pub platform: TargetPlatform,
    pub product_type: ProductType,
    pub bundle_identifier: Option<String>,
    pub source_files: Vec<PathBuf>,
    pub header_files: Vec<PathBuf>,
    pub resource_files: Vec<PathBuf>,
    pub frameworks: Vec<String>,
    pub build_settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfiguration {
    pub name: String, // e.g., "Debug", "Release", "Profile"
    pub mode: BuildMode,
    pub optimization_level: String, // e.g., "-O0", "-O3", "-Osize"
    pub defines: Vec<String>,
    pub swift_flags: Vec<String>,
    pub c_flags: Vec<String>,
    pub ld_flags: Vec<String>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scheme {
    pub name: String,
    pub build_target_id: String,
    pub default_configuration: String,
    pub test_target_id: Option<String>,
    pub launch_arguments: Vec<String>,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainInfo {
    pub id: String,
    pub name: String,
    pub binary_path: Option<PathBuf>,
    pub version: Option<String>,
    pub status: CapabilityStatus,
    pub triple: Option<String>,
    pub details: Option<String>,
    pub documentation_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildEvent {
    StageStarted { stage_name: String, phase: BuildPhase },
    StageOutput { stage_name: String, stream: String, text: String },
    StageCompleted { stage_name: String, duration_ms: u64, success: bool },
    DiagnosticEmitted { diagnostic: Diagnostic },
    BuildProgress { fraction: f32, message: String },
    BuildFinished { report: BuildReport },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Error => write!(f, "error"),
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Information => write!(f, "info"),
            DiagnosticSeverity::Hint => write!(f, "hint"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub id: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: String, // e.g. "clang", "swiftc", "dart", "ld64"
    pub code: Option<String>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256_hash: String,
    pub platform: TargetPlatform,
    pub is_directory_bundle: bool,
    pub bundle_identifier: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugState {
    Configuring,
    Running,
    Paused,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
    pub condition: Option<String>,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub id: String,
    pub project_id: String,
    pub target_id: String,
    pub state: DebugState,
    pub process_id: Option<u32>,
    pub active_breakpoints: Vec<Breakpoint>,
    pub adapter_name: String, // e.g. "lldb", "flutter-dap"
    pub started_at: DateTime<Utc>,
}

