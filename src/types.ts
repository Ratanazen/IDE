export type NavTab = 'projects' | 'builds' | 'artifacts' | 'toolchain' | 'settings';

export type BuildTarget = 'web' | 'linux' | 'apk' | 'appbundle' | 'ios';
export type BuildMode = 'debug' | 'profile' | 'release';

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
