export type NavTab = 'projects' | 'editor' | 'builds' | 'artifacts' | 'toolchain' | 'settings';

export type BuildTarget = 'web' | 'linux' | 'apk' | 'appbundle' | 'ios';
export type BuildMode = 'debug' | 'profile' | 'release';
export type BuildPhase =
  | 'Idle'
  | 'Validating'
  | 'ResolvingDependencies'
  | 'Building'
  | 'VerifyingArtifact'
  | 'Succeeded'
  | 'Failed'
  | 'Cancelled';

export interface ToolStatus {
  name: string;
  available: boolean;
  version?: string;
  path?: string;
  details?: string;
}

export interface AndroidSdkInfo {
  path: string;
  platforms: string[];
  build_tools: string[];
  has_cmdline_tools: boolean;
  licenses_accepted: boolean;
}

export interface JdkInfo {
  version: string;
  path: string;
  is_agp_compatible: boolean;
}

export interface ToolchainReport {
  flutter: ToolStatus;
  dart: ToolStatus;
  android?: AndroidSdkInfo;
  jdks: JdkInfo[];
  linux_deps: Record<string, ToolStatus>;
  ready_for_web: boolean;
  ready_for_linux: boolean;
  ready_for_android: boolean;
  diagnostics: string[];
}

export interface ProjectMetadata {
  name: string;
  path: string;
  description?: string;
  version?: string;
  platforms: string[];
  dependencies: string[];
  dev_dependencies: string[];
}

export interface CreateProjectRequest {
  name: string;
  org?: string;
  parent_dir: string;
  platforms: string[];
}

export interface ProjectActionResult {
  success: boolean;
  exit_code?: number;
  stdout: string;
  stderr: string;
}

export interface DeviceCapabilities {
  hotReload: boolean;
  hotRestart: boolean;
  screenshot: boolean;
  flutterExit: boolean;
}

export interface Device {
  id: string;
  name: string;
  isSupported: boolean;
  targetPlatform: string;
  emulator: boolean;
  sdk?: string;
  capabilities?: DeviceCapabilities;
}

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size_bytes: number;
}

export interface BuildResult {
  job_id: string;
  project_path: string;
  target: string;
  mode: string;
  status: BuildPhase;
  exit_code?: number;
  artifact_paths: string[];
  log_path?: string;
  error_code?: string;
  started_at: string;
  finished_at?: string;
}

export type IosCapabilityStatus =
  | 'Verified'
  | 'Experimental'
  | 'Unsupported'
  | 'Blocked'
  | 'NotTested'
  | 'MissingDependency';

export interface IosToolchainReport {
  host_os: string;
  flutter_version?: string;
  swift_version?: string;
  clang_version?: string;
  can_compile: IosCapabilityStatus;
  can_package_ipa: IosCapabilityStatus;
  can_sign: IosCapabilityStatus;
  can_install_to_device: IosCapabilityStatus;
  limitations: string[];
}

export type IpaValidationStatus =
  | 'PackageCreated'
  | 'PackageValidated'
  | 'SigningVerified'
  | 'InstallTestPassed'
  | 'DistributionReady'
  | 'PackageCreatedButNotFullyVerified';

export interface IpaValidationReport {
  status: IpaValidationStatus;
  app_name?: string;
  bundle_id?: string;
  version?: string;
  has_payload_dir: boolean;
  has_app_bundle: boolean;
  has_code_signature: boolean;
  has_info_plist: boolean;
  details: string[];
}

// ---------------------------------------------------------------------------
// Ecosystem Models (VS Code + Xcode Hybrid Architecture)
// ---------------------------------------------------------------------------

export type ProjectKind = 'Flutter' | 'SwiftPackage' | 'XcodeProject' | 'ClangNative' | 'Unknown';

export type ProductType =
  | 'Application'
  | 'Framework'
  | 'StaticLibrary'
  | 'DynamicLibrary'
  | 'CommandLineTool'
  | 'UnitTestBundle';

export interface BuildTargetModel {
  id: string;
  name: string;
  platform: string;
  product_type: ProductType;
  bundle_identifier?: string;
  source_files: string[];
  header_files: string[];
  resource_files: string[];
  frameworks: string[];
  build_settings: Record<string, string>;
}

export interface BuildConfigurationModel {
  name: string;
  mode: BuildMode;
  optimization_level: string;
  defines: string[];
  swift_flags: string[];
  c_flags: string[];
  ld_flags: string[];
  environment: Record<string, string>;
}

export interface SchemeModel {
  name: string;
  build_target_id: string;
  default_configuration: string;
  test_target_id?: string;
  launch_arguments: string[];
  environment_variables: Record<string, string>;
}

export interface EcosystemProject {
  id: string;
  name: string;
  path: string;
  kind: ProjectKind;
  root_directory: string;
  active_scheme?: string;
  active_configuration?: string;
  active_target?: string;
  targets: BuildTargetModel[];
  schemes: SchemeModel[];
  configurations: BuildConfigurationModel[];
  deployment_target?: string;
  bundle_identifier?: string;
}

export type DiagnosticSeverity = 'error' | 'warning' | 'info' | 'hint';

export interface Diagnostic {
  id: string;
  file_path: string;
  line: number;
  column: number;
  end_line?: number;
  end_column?: number;
  severity: DiagnosticSeverity;
  message: string;
  source: string;
  code?: string;
  snippet?: string;
}

export interface ToolchainInfo {
  id: string;
  name: string;
  binary_path?: string;
  version?: string;
  status: IosCapabilityStatus;
  triple?: string;
  details?: string;
  documentation_url?: string;
}

export interface LspServerInfo {
  id: string;
  name: string;
  path?: string;
  supported_languages: string[];
  status: IosCapabilityStatus;
  command_args: string[];
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  permissions: string[];
  activation_events: string[];
  is_enabled: boolean;
  is_builtin: boolean;
}

export type BottomPanelTab = 'problems' | 'output' | 'debug' | 'terminal';

export type ActivityBarTab =
  | 'explorer'
  | 'xcode-inspector'
  | 'search'
  | 'git'
  | 'debug'
  | 'toolchain'
  | 'plugins'
  | 'settings';

