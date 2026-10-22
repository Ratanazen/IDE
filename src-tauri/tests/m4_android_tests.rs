use ratana_flutter_studio_lib::artifacts::ArtifactVerifier;
use ratana_flutter_studio_lib::builds::backends::FlutterAndroidBackend;
use ratana_flutter_studio_lib::builds::Target;
use ratana_flutter_studio_lib::toolchain::ToolchainDetector;
use std::path::PathBuf;

#[test]
fn test_android_release_signing_detection() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.parent().unwrap().join("fixtures/fixture_app");

    // By default, fixture app has no custom key.properties
    let has_signing = FlutterAndroidBackend::has_release_signing(&fixture_path);
    assert!(!has_signing, "Fresh fixture app should not have key.properties");
}

#[test]
fn test_android_apk_artifact_verification() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let apk_path = manifest_dir
        .parent()
        .unwrap()
        .join("fixtures/fixture_app/build/app/outputs/flutter-apk/app-release.apk");

    assert!(apk_path.exists(), "app-release.apk must exist on disk");

    let verified = ArtifactVerifier::verify_file(&apk_path, Target::AndroidApk)
        .expect("Verification of real release APK must succeed");

    assert_eq!(verified.name, "app-release.apk");
    assert!(verified.size_bytes > 1_000_000, "APK must be a real non-empty binary (> 1MB)");
    assert!(!verified.sha256_hash.is_empty(), "SHA-256 hash must be computed");
}

#[test]
fn test_missing_cmdline_tools_instruction() {
    let report = ToolchainDetector::generate_report();

    // On this host, cmdline-tools is missing
    if let Some(ref android) = report.android {
        if !android.has_cmdline_tools {
            let has_instruction = report.diagnostics.iter().any(|d| {
                d.contains("cmdline-tools") && d.contains("Install via AUR")
            });
            assert!(
                has_instruction,
                "A missing toolchain component must produce clear actionable instructions, not silence!"
            );
        }
    }
}
