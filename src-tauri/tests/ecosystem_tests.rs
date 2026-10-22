use ratana_flutter_studio_lib::compiler::{ClangInvocationBuilder, Ld64InvocationBuilder};
use ratana_flutter_studio_lib::core::models::{
    CapabilityStatus, DiagnosticSeverity, ProductType, ProjectKind,
};
use ratana_flutter_studio_lib::debug::{
    IosDeviceDeployment, LaunchConfiguration, LldbAdapter,
};
use ratana_flutter_studio_lib::language::DiagnosticsParser;
use ratana_flutter_studio_lib::plugins::{PluginPermission, PluginRegistry};
use ratana_flutter_studio_lib::projects::{
    ProjectManager, SwiftPackageManager, XcodeProjectParser,
};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_spm_package_parser_integration() {
    let manifest = r#"
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "RiduCoreiOS",
    platforms: [
        .iOS(.v16)
    ],
    products: [
        .library(name: "RiduCoreiOS", targets: ["RiduCoreiOS"]),
    ],
    targets: [
        .target(name: "RiduCoreiOS", dependencies: []),
        .testTarget(name: "RiduCoreiOSTests", dependencies: ["RiduCoreiOS"]),
    ]
)
"#;

    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Package.swift"), manifest).unwrap();

    let project = SwiftPackageManager::parse_package(dir.path()).expect("Should parse SPM package");
    assert_eq!(project.name, "RiduCoreiOS");
    assert_eq!(project.kind, ProjectKind::SwiftPackage);
    assert_eq!(project.deployment_target.as_deref(), Some("16"));
    assert_eq!(project.targets.len(), 2);
    assert_eq!(project.targets[0].name, "RiduCoreiOS");
    assert_eq!(project.targets[0].product_type, ProductType::Framework);
    assert_eq!(project.targets[1].name, "RiduCoreiOSTests");
    assert_eq!(project.targets[1].product_type, ProductType::UnitTestBundle);
}

#[test]
fn test_xcode_pbxproj_parser_integration() {
    let sample = r#"
// !$*UTF8*$!
{
	archiveVersion = 1;
	classes = {
	};
	objectVersion = 54;
	objects = {
/* Begin PBXNativeTarget section */
		97C146ED1CF9000F007C117D /* NativeApp */ = {
			isa = PBXNativeTarget;
			buildConfigurationList = 97C147051CF9000F007C117D;
			name = NativeApp;
			productName = NativeApp;
			productReference = 97C146EE1CF9000F007C117D /* NativeApp.app */;
			productType = "com.apple.product-type.application";
		};
/* End PBXNativeTarget section */
/* Begin XCBuildConfiguration section */
		97C147061CF9000F007C117D /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				IPHONEOS_DEPLOYMENT_TARGET = 15.0;
				PRODUCT_BUNDLE_IDENTIFIER = com.ridu.nativeapp;
				SWIFT_VERSION = 5.9;
			};
			name = Debug;
		};
		97C147071CF9000F007C117D /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				IPHONEOS_DEPLOYMENT_TARGET = 15.0;
				PRODUCT_BUNDLE_IDENTIFIER = com.ridu.nativeapp;
				SWIFT_VERSION = 5.9;
			};
			name = Release;
		};
/* End XCBuildConfiguration section */
	};
}
"#;

    let dir = tempfile::tempdir().unwrap();
    let xcode_dir = dir.path().join("NativeApp.xcodeproj");
    fs::create_dir_all(&xcode_dir).unwrap();
    fs::write(xcode_dir.join("project.pbxproj"), sample).unwrap();

    let project = XcodeProjectParser::parse_project(dir.path()).expect("Should parse Xcode project");
    assert_eq!(project.name, "NativeApp");
    assert_eq!(project.kind, ProjectKind::XcodeProject);
    assert_eq!(project.deployment_target.as_deref(), Some("15.0"));
    assert_eq!(project.bundle_identifier.as_deref(), Some("com.ridu.nativeapp"));
    assert_eq!(project.targets.len(), 1);
    assert_eq!(project.targets[0].name, "NativeApp");
    assert_eq!(project.targets[0].product_type, ProductType::Application);
    assert_eq!(project.configurations.len(), 2);
    assert_eq!(project.schemes.len(), 1);
}

#[test]
fn test_inspect_ecosystem_project_on_fixture() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.parent().unwrap().join("fixtures/fixture_app");

    let project = ProjectManager::inspect_ecosystem_project(&fixture_path)
        .expect("Should inspect fixture app");

    assert_eq!(project.name, "fixture_app");
    assert_eq!(project.kind, ProjectKind::Flutter);
    assert!(!project.targets.is_empty());
    assert!(!project.schemes.is_empty());
    assert!(!project.configurations.is_empty());

    let target_names: Vec<_> = project.targets.iter().map(|t| t.name.as_str()).collect();
    assert!(target_names.iter().any(|n| n.contains("linux")));
    assert!(target_names.iter().any(|n| n.contains("web")));
    assert!(target_names.iter().any(|n| n.contains("android")));
}

#[test]
fn test_clang_and_ld64_argument_builders() {
    let clang_builder = ClangInvocationBuilder::new("arm64-apple-ios16.0")
        .sysroot("/opt/iOS-SDKs/iPhoneOS16.0.sdk")
        .add_include("/opt/iOS-SDKs/iPhoneOS16.0.sdk/usr/include")
        .add_framework_path("/opt/iOS-SDKs/iPhoneOS16.0.sdk/System/Library/Frameworks")
        .add_define("TARGET_OS_IPHONE=1");

    let clang_cmd = clang_builder.build_command(Path::new("main.m"), Path::new("build/main.o"));
    assert_eq!(clang_cmd.program.to_string_lossy(), "clang");
    assert!(clang_cmd.args.contains(&"-target".to_string()));
    assert!(clang_cmd.args.contains(&"arm64-apple-ios16.0".to_string()));
    assert!(clang_cmd.args.contains(&"-fobjc-arc".to_string()));
    assert!(clang_cmd.args.contains(&"-fmodules".to_string()));

    let ld64_builder = Ld64InvocationBuilder::new("arm64", "16.0")
        .syslibroot("/opt/iOS-SDKs/iPhoneOS16.0.sdk")
        .add_framework("UIKit")
        .add_framework("Foundation")
        .add_rpath("@executable_path/Frameworks");

    let ld64_cmd = ld64_builder.build_command(
        &[PathBuf::from("build/main.o")],
        Path::new("build/App.app/App"),
    );

    assert!(ld64_cmd.program.to_string_lossy().contains("ld64.lld"));
    assert!(ld64_cmd.args.contains(&"-arch".to_string()));
    assert!(ld64_cmd.args.contains(&"arm64".to_string()));
    assert!(ld64_cmd.args.contains(&"-framework".to_string()));
    assert!(ld64_cmd.args.contains(&"UIKit".to_string()));
}

#[test]
fn test_compiler_diagnostics_parser() {
    let sample = r#"
Sources/App.swift:12:8: error: no such module 'SwiftUI'
Sources/AppDelegate.m:45:12: warning: unused variable 'token'
Sources/main.c:5:1: note: previous definition is here
"#;

    let diags = DiagnosticsParser::parse(sample, "clang");
    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
    assert_eq!(diags[0].line, 12);
    assert_eq!(diags[0].column, 8);
    assert!(diags[0].message.contains("no such module 'SwiftUI'"));

    assert_eq!(diags[1].severity, DiagnosticSeverity::Warning);
    assert_eq!(diags[1].line, 45);

    assert_eq!(diags[2].severity, DiagnosticSeverity::Information);
    assert_eq!(diags[2].line, 5);
}

#[test]
fn test_lldb_and_device_tools() {
    let (status, tools) = IosDeviceDeployment::probe_idevice_tools();
    assert_eq!(status, CapabilityStatus::Verified);
    assert!(tools.iter().any(|t| t.contains("ideviceinfo")));

    let config = LaunchConfiguration {
        name: "Debug Runner".to_string(),
        request: "launch".to_string(),
        program: PathBuf::from("build/Runner.app/Runner"),
        args: vec!["--observe".to_string()],
        cwd: None,
        env: HashMap::new(),
        stop_at_entry: false,
        target_architecture: Some("arm64".to_string()),
    };

    let script = LldbAdapter::generate_script(&config, &[]);
    assert!(script.contains("target create \"build/Runner.app/Runner\""));
    assert!(script.contains("settings set target.run-args \"--observe\""));
}

#[test]
fn test_plugins_and_permissions() {
    let registry = PluginRegistry::new();
    let plugins = registry.list_plugins();
    assert_eq!(plugins.len(), 4);

    let swift_plugin = plugins.iter().find(|p| p.id == "ridu.apple-swift").unwrap();
    assert!(swift_plugin.is_enabled);
    assert!(swift_plugin.permissions.contains(&PluginPermission::ToolchainAccess));

    let xcode_plugin = plugins.iter().find(|p| p.id == "ridu.xcode-inspector").unwrap();
    assert!(xcode_plugin.is_enabled);
}
