use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiduError {
    #[error("Toolchain error: {0}")]
    Toolchain(#[from] ToolchainError),

    #[error("Project error: {0}")]
    Project(#[from] ProjectError),

    #[error("Build plan error: {0}")]
    BuildPlan(#[from] BuildPlanError),

    #[error("Build stage error: {0}")]
    Stage(#[from] StageError),

    #[error("Process execution error: {0}")]
    Process(#[from] ProcessError),

    #[error("iOS toolchain error: {0}")]
    Ios(#[from] IosError),

    #[error("Signing error: {0}")]
    Signing(#[from] SigningError),

    #[error("IPA validation error: {0}")]
    IpaValidation(#[from] IpaValidationError),

    #[error("Security violation: {0}")]
    Security(#[from] SecurityError),

    #[error("Internal I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ToolchainError {
    #[error("Binary '{0}' was not found in system PATH")]
    BinaryNotFound(String),

    #[error("Command execution failed for tool '{0}': {1}")]
    ExecutionFailed(String, String),

    #[error("Failed to parse tool output for '{0}': {1}")]
    ParseError(String, String),

    #[error("Missing dependency: {0}")]
    MissingDependency(String),
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Invalid project name '{0}': {1}")]
    InvalidName(String, String),

    #[error("Invalid project path '{0}': {1}")]
    InvalidPath(String, String),

    #[error("Missing pubspec.yaml in project: {0}")]
    MissingPubspec(String),

    #[error("Action execution error: {0}")]
    ActionFailed(String),

    #[error("Filesystem error: {0}")]
    FsError(String),
}

#[derive(Debug, Error)]
pub enum BuildPlanError {
    #[error("Unsupported target platform '{0}' for current host")]
    UnsupportedTarget(String),

    #[error("Invalid build mode '{0}'")]
    InvalidMode(String),

    #[error("Prerequisite check failed for plan '{0}': {1}")]
    PrerequisiteFailed(String, String),

    #[error("Project build lock acquisition failed: {0}")]
    LockContention(String),
}

#[derive(Debug, Error)]
pub enum StageError {
    #[error("Stage '{0}' failed with exit code {1:?}: {2}")]
    ExecutionFailed(String, Option<i32>, String),

    #[error("Stage '{0}' timed out after {1} seconds")]
    Timeout(String, u64),

    #[error("Stage '{0}' was cancelled by user")]
    Cancelled(String),

    #[error("Stage '{0}' unsupported on current environment: {1}")]
    Unsupported(String, String),
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Failed to spawn process '{0}': {1}")]
    SpawnFailed(String, String),

    #[error("Process timed out: {0}")]
    TimedOut(String),

    #[error("Process was cancelled: {0}")]
    Cancelled(String),

    #[error("Failed to kill process group {0}: {1}")]
    KillFailed(i32, String),

    #[error("IO error during execution: {0}")]
    Io(String),
}

#[derive(Debug, Error)]
pub enum IosError {
    #[error("Invalid iOS directory structure in {0}")]
    InvalidProjectStructure(String),

    #[error("Plugin incompatibility: {0}")]
    PluginIncompatible(String),

    #[error("Dart AOT compilation error: {0}")]
    DartCompilation(String),

    #[error("Native compilation error: {0}")]
    NativeCompilation(String),

    #[error("Linker error: {0}")]
    Linker(String),

    #[error("Packaging error: {0}")]
    Packaging(String),

    #[error("Platform unsupported on Linux: {0}")]
    UnsupportedOnLinux(String),
}

#[derive(Debug, Error)]
pub enum SigningError {
    #[error("Signing certificate not found: {0}")]
    CertificateNotFound(String),

    #[error("Provisioning profile invalid: {0}")]
    InvalidProvisioningProfile(String),

    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),

    #[error("Signing tool not available on this platform: {0}")]
    ToolUnavailable(String),
}

#[derive(Debug, Error)]
pub enum IpaValidationError {
    #[error("Archive is empty or unreadable: {0}")]
    UnreadableArchive(String),

    #[error("Invalid archive format: {0}")]
    InvalidFormat(String),

    #[error("Missing Payload/ directory in IPA")]
    MissingPayloadDirectory,

    #[error("Missing .app bundle inside Payload/")]
    MissingAppBundle,

    #[error("Missing Info.plist in application bundle")]
    MissingInfoPlist,

    #[error("Corrupt or invalid Info.plist: {0}")]
    InvalidInfoPlist(String),

    #[error("Code signature invalid: {0}")]
    InvalidSignature(String),
}

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Path traversal detected: {0}")]
    PathTraversal(String),

    #[error("Access outside allowed workspace directory: {0}")]
    OutsideWorkspace(String),

    #[error("Invalid symlink target: {0}")]
    DangerousSymlink(String),
}
