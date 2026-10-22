use ratana_flutter_studio_lib::builds::pipeline::{BuildContext, BuildPipeline};
use ratana_flutter_studio_lib::core::models::{
    BuildMode, BuildRequest, CapabilityStatus, TargetPlatform,
};
use ratana_flutter_studio_lib::ios::engine::IosBuildEngine;
use ratana_flutter_studio_lib::ios::inspector::IosProjectInspector;
use ratana_flutter_studio_lib::ios::macho::{
    MachOInfo, CPU_TYPE_ARM64, LC_BUILD_VERSION, LC_CODE_SIGNATURE, MH_MAGIC_64,
};
use ratana_flutter_studio_lib::signing::profile::ProvisioningProfileParser;
use ratana_flutter_studio_lib::signing::verifier::CodeSignatureVerifier;
use ratana_flutter_studio_lib::toolchain::ToolchainDetector;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_toolchain_detection_apis() {
    let flutter = ToolchainDetector::detect_flutter_sdk().expect("Flutter SDK must be detected");
    assert!(flutter.available);
    assert!(flutter.path.is_some());

    let dart = ToolchainDetector::detect_dart_sdk().expect("Dart SDK must be detected");
    assert!(dart.available);
    assert!(dart.version.is_some());

    let clang = ToolchainDetector::detect_clang_toolchain().expect("Clang must be detected");
    assert!(clang.available);

    let swift = ToolchainDetector::detect_swift_toolchain().expect("Swift check should succeed");
    // Swift may or may not be installed on this host
    let _ = swift.available;

    let sdks = ToolchainDetector::detect_available_ios_sdks().expect("iOS SDK scan should succeed");
    let _ = sdks.len();

    let cap = ToolchainDetector::generate_capability_report();
    assert_eq!(cap.web, CapabilityStatus::Verified);
    assert_eq!(cap.linux_desktop, CapabilityStatus::Verified);
    assert_eq!(cap.android_apk, CapabilityStatus::Verified);
    assert_eq!(cap.ios_app, CapabilityStatus::Blocked);
}

#[test]
fn test_macho_64bit_parser() {
    // Construct a binary with Mach-O 64-bit header
    let mut bytes = Vec::new();

    // 1. Magic (MH_MAGIC_64)
    bytes.extend_from_slice(&MH_MAGIC_64.to_le_bytes());
    // 2. cputype (CPU_TYPE_ARM64)
    bytes.extend_from_slice(&CPU_TYPE_ARM64.to_le_bytes());
    // 3. cpusubtype (0)
    bytes.extend_from_slice(&0u32.to_le_bytes());
    // 4. filetype (MH_EXECUTE = 2)
    bytes.extend_from_slice(&2u32.to_le_bytes());
    // 5. ncmds (2)
    bytes.extend_from_slice(&2u32.to_le_bytes());
    // 6. sizeofcmds (32)
    bytes.extend_from_slice(&32u32.to_le_bytes());
    // 7. flags (0)
    bytes.extend_from_slice(&0u32.to_le_bytes());
    // 8. reserved (0)
    bytes.extend_from_slice(&0u32.to_le_bytes());

    // Command 1: LC_BUILD_VERSION (cmd=0x32, cmdsize=16)
    bytes.extend_from_slice(&LC_BUILD_VERSION.to_le_bytes());
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes()); // padding

    // Command 2: LC_CODE_SIGNATURE (cmd=0x1d, cmdsize=16)
    bytes.extend_from_slice(&LC_CODE_SIGNATURE.to_le_bytes());
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes()); // padding

    let info = MachOInfo::parse_from_bytes(&bytes).expect("Should parse valid Mach-O binary");
    assert!(info.is_64bit);
    assert!(info.is_arm64);
    assert!(info.has_build_version_command);
    assert!(info.has_code_signature_command);
    assert_eq!(info.num_commands, 2);

    // Test rejection of non-Mach-O bytes
    let invalid_bytes = vec![0x7f, 0x45, 0x4c, 0x46]; // ELF header
    assert!(MachOInfo::parse_from_bytes(&invalid_bytes).is_err());
}

#[test]
fn test_provisioning_profile_parsing() {
    let sample_plist = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
        <key>AppIDName</key>
        <string>Ridu Demo App</string>
        <key>Name</key>
        <string>Ridu Development Profile</string>
        <key>TeamName</key>
        <string>Ridu Zen Team</string>
        <key>TeamIdentifier</key>
        <string>ABC123XYZ</string>
        <key>UUID</key>
        <string>12345678-1234-1234-1234-123456789012</string>
        <key>ProvisionsAllDevices</key>
        <true/>
    </dict>
    </plist>
    "#;

    // Wrap with dummy CMS wrapper bytes
    let mut profile_data = b"MIIB...CMS_HEADER...".to_vec();
    profile_data.extend_from_slice(sample_plist.as_bytes());
    profile_data.extend_from_slice(b"...CMS_TRAILER...");

    let info = ProvisioningProfileParser::parse_from_bytes(&profile_data)
        .expect("Should parse provisioning profile from bytes");

    assert_eq!(info.name.as_deref(), Some("Ridu Development Profile"));
    assert_eq!(info.team_name.as_deref(), Some("Ridu Zen Team"));
    assert_eq!(info.team_identifier.as_deref(), Some("ABC123XYZ"));
    assert!(info.provisions_all_devices);
}

#[test]
fn test_code_signature_verifier_on_bundle() {
    let temp = tempdir().expect("Failed to create tempdir");
    let app_bundle = temp.path().join("TestApp.app");
    fs::create_dir_all(&app_bundle).unwrap();

    // 1. Unsigned bundle
    let res = CodeSignatureVerifier::verify_bundle(&app_bundle).expect("Verification should succeed");
    assert!(!res.is_signed);
    assert!(res.authority.is_none());

    // 2. Ad-hoc signed bundle with _CodeSignature/CodeResources
    let sig_dir = app_bundle.join("_CodeSignature");
    fs::create_dir_all(&sig_dir).unwrap();
    fs::write(sig_dir.join("CodeResources"), "<plist><dict></dict></plist>").unwrap();

    let res2 = CodeSignatureVerifier::verify_bundle(&app_bundle).expect("Verification should succeed");
    assert!(res2.is_signed);
    assert!(res2.is_ad_hoc);
}

#[test]
fn test_ios_project_inspection_and_environment() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.parent().unwrap().join("fixtures/fixture_app");
    assert!(fixture_path.exists());

    let info = IosProjectInspector::inspect_ios_project(&fixture_path).expect("Should inspect iOS project");
    assert_eq!(info.app_name.as_deref(), Some("fixture_app"));

    let plugins = IosProjectInspector::validate_ios_plugins(&fixture_path).expect("Should inspect plugins");
    let _ = plugins.len();

    let ctx = BuildContext {
        project_path: fixture_path.clone(),
        target: TargetPlatform::IosApp,
        mode: BuildMode::Release,
        env: HashMap::new(),
    };

    let env = IosBuildEngine::prepare_ios_build_environment(&ctx).expect("Should prepare env");
    assert_eq!(env.target_triple, "arm64-apple-ios");

    let dart_res = IosBuildEngine::compile_flutter_dart(&ctx).expect("Should execute compile_flutter_dart");
    assert_eq!(dart_res.status, CapabilityStatus::Blocked);
    assert!(dart_res.message.contains("Darwin gen_snapshot is unavailable on Linux"));
}

#[tokio::test]
async fn test_build_planning_and_stage_execution() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.parent().unwrap().join("fixtures/fixture_app");

    let req = BuildRequest {
        project_path: fixture_path.clone(),
        target: TargetPlatform::LinuxDesktop,
        mode: BuildMode::Release,
        clean_before_build: true,
        run_analyze: true,
        run_tests: true,
    };

    let plan = BuildPipeline::create_build_plan(&req);
    assert_eq!(plan.project_name, "fixture_app");
    assert_eq!(plan.stages.len(), 7);

    // Validate prerequisites
    assert!(BuildPipeline::validate_build_prerequisites(&plan).is_ok());

    let ctx = BuildContext {
        project_path: fixture_path,
        target: TargetPlatform::LinuxDesktop,
        mode: BuildMode::Release,
        env: HashMap::new(),
    };

    let stage_res = BuildPipeline::execute_build_stage(&plan.stages[0], &ctx).await
        .expect("Prerequisites stage should succeed");
    assert!(stage_res.success);
    assert_eq!(stage_res.stage_name, "validate_prerequisites");
}
