# Ratana Flutter Studio: Architecture & System Design

## 1. System Overview

**Ratana Flutter Studio** is a dedicated Linux desktop application (targeting Arch Linux, Archcraft, and rolling distributions like Garuda Linux) designed to manage, build, test, package, and diagnose Flutter applications using the real Flutter CLI, Dart SDK, Android SDK, and Linux desktop toolchains without relying on Android Studio, VS Code, or web wrappers.

### Priority Order:
1. **Real Flutter build engine** for Web, Linux Desktop, Android APK, and Android AAB.
2. **Logs, build history, and artifact registry**.
3. **AI build assistant** (diagnosis, patch generation, one-click rollback).
4. **Experimental, evidence-driven iOS toolchain module**.

---

## 2. Technology Stack

- **Desktop Shell**: Tauri 2 (Rust backend + Webview frontend).
- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS, Lucide icons.
- **Backend**: Rust 2021 edition (`src-tauri`).
- **Flutter Integration**: Real `flutter` and `dart` CLIs. Never fake project structures.
- **Local AI Provider**: Ollama REST API adapter (`http://localhost:11434`).
- **Remote AI Provider**: Provider-neutral HTTP adapter interface.
- **Target OS**: Linux x86_64 (Arch Linux, Archcraft, Garuda Linux).

---

## 3. Core Architectural Principles & Honesty Rules

1. **Separation of Concerns**: The frontend contains **zero build logic**. It only dispatches typed commands to the Rust backend and listens to streaming events.
2. **No Command Concatenation**: Executables and arguments are never constructed via string concatenation or interactive shell wrappers (`sh -c`). Commands are constructed as an explicit executable path plus an array of argument strings (`std::process::Command::new(binary).args(&[...])`).
3. **Process Group Management**: Processes are launched with independent process groups (`setpgid(0, 0)`). Cancellation targets the entire process group (`killpg(SIGTERM)` followed by `killpg(SIGKILL)` after a 2000ms grace period) to eliminate orphan child processes (such as Gradle daemons or child compilers).
4. **Per-Project Build Mutex**: Each project path has an associated in-memory lock (`Arc<Mutex<HashMap<PathBuf, BuildLock>>>`). Exactly one build job may be active per project at any given time.
5. **Truth in Capabilities**: Every capability has exactly one status: `VERIFIED`, `EXPERIMENTAL`, `UNSUPPORTED`, `BLOCKED`, or `NOT_TESTED`. A binary existing on `PATH` does not make a capability `VERIFIED`. Mocks and fixtures are restricted to unit tests and never reported as real build results.
6. **Artifact Discovery & Integrity**: Never assume hardcoded artifact locations. Parse Flutter's `Built ...` stdout output, fall back to directory scanning, verify non-empty existence, and compute SHA-256 hashes.

---

## 4. Backend Module Structure (`src-tauri/src/`)

```
src-tauri/
├── Cargo.toml                   # Workspace root manifest
├── crates/
│   ├── ridu-core/               # Shared types, TargetPlatform, BuildPhase, StageStatus, error types
│   ├── ridu-security/           # PathValidator, PathSecurityError, secret scrubbing regexes
│   ├── ridu-toolchain/          # ToolchainDetector (Flutter, Dart, Clang, Swift, Android, iOS SDKs)
│   ├── ridu-project/            # ProjectManager (flutter create, import, analyze, format, pubspec)
│   ├── ridu-build/              # BuildPipeline, ProcessRunner (setpgid/killpg), concurrency locks
│   ├── ridu-ios/                # Original iOS engine, Mach-O parser, plugins validator, environment
│   ├── ridu-signing/            # CodeSignatureVerifier, ProvisioningProfileParser
│   ├── ridu-artifacts/          # ArtifactDiscovery, ArtifactVerifier, IpaValidator, BuildHistoryStore
│   └── ridu-device/             # DeviceManager (flutter devices --machine)
├── src/
│   ├── lib.rs                   # Tauri application setup, state, command registry
│   └── main.rs                  # Entrypoint
└── tests/                       # Integration test suites
```

---

## 5. Build Job Lifecycle & Data Model

### 5.1 Lifecycle States
```
               +-------------------------------------------------+
               |                      IDLE                       |
               +-------------------------------------------------+
                                        | (Dispatch Build)
                                        v
               +-------------------------------------------------+
               |                   VALIDATING                    |
               +-------------------------------------------------+
                                        |
                                        v
               +-------------------------------------------------+
               |              RESOLVING_DEPENDENCIES             |
               |             (flutter pub get, etc.)             |
               +-------------------------------------------------+
                                        |
                                        v
               +-------------------------------------------------+
               |                    BUILDING                     |
               |        (flutter build <target> <mode>)          |
               +-------------------------------------------------+
                                        |
                                        v
               +-------------------------------------------------+
               |               VERIFYING_ARTIFACT                |
               |          (disk check, hash, size)               |
               +-------------------------------------------------+
                               /        |        \
                             /          |          \
                            v           v           v
                    +-----------+ +-----------+ +-----------+
                    | SUCCEEDED | |  FAILED   | | CANCELLED |
                    +-----------+ +-----------+ +-----------+
```

### 5.2 Core Data Structures
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    Web,
    Linux,
    AndroidApk,
    AndroidAab,
    Ios,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildMode {
    Debug,
    Profile,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequest {
    pub project_path: PathBuf,
    pub target: Target,
    pub mode: BuildMode,
    pub clean_before_build: bool,
    pub run_analyze: bool,
    pub run_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub job_id: String,
    pub project_path: PathBuf,
    pub target: Target,
    pub mode: BuildMode,
    pub status: BuildPhase,
    pub exit_code: Option<i32>,
    pub artifact_paths: Vec<PathBuf>,
    pub log_path: Option<PathBuf>,
    pub error_code: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

---

## 6. IPC Protocol & Security Boundaries

1. **Frontend-to-Backend (Commands)**:
   - `get_toolchain_report`: Scans host environment and returns detection status.
   - `list_projects`: Returns known projects and cached metadata.
   - `create_project`: Invokes `flutter create` with validated parameters.
   - `start_build`: Validates request, checks lock, starts asynchronous build job.
   - `cancel_build`: Sends cancellation signal to project's active process group.
   - `get_build_history`: Reads persisted job history.
   - `open_output_dir`: Opens artifact folder in native file manager (`xdg-open`).
2. **Backend-to-Frontend (Events)**:
   - `build-phase-changed`: `{ jobId, phase, timestamp }`
   - `build-stdout`: `{ jobId, line, timestamp }`
   - `build-stderr`: `{ jobId, line, timestamp }`
   - `build-finished`: `{ jobId, result }`
3. **Security Safeguards**:
   - Tauri shell plugin is omitted to eliminate arbitrary process spawning.
   - All file paths are strictly canonicalized (`dunce::canonicalize`).
   - Project paths must exist and contain a valid `pubspec.yaml`.
   - Sensitive strings matching token/password/keystore patterns are redacted before log persistence or websocket streaming.
