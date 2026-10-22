use ratana_flutter_studio_lib::ios::{
    IosCapabilityStatus, IosToolchainDetector, IpaValidationError, IpaValidationStatus,
    IpaValidator,
};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn test_ios_toolchain_detector_honesty_on_linux() {
    let report = IosToolchainDetector::generate_report();

    if report.host_os == "linux" {
        // Must NEVER claim full verification on Linux without evidence!
        assert_ne!(
            report.can_compile,
            IosCapabilityStatus::Verified,
            "Compilation must not be verified on Linux without a macOS host"
        );

        assert!(
            report.limitations.iter().any(|l| l.contains("macOS and Xcode")),
            "Report must explicitly document the macOS and Xcode requirement"
        );
    }
}

#[test]
fn test_ipa_validator_rejects_invalid_fixtures() {
    let temp = tempdir().expect("Failed to create tempdir");

    // 1. Missing file
    let missing_path = temp.path().join("missing.ipa");
    let err = IpaValidator::validate_ipa(&missing_path).unwrap_err();
    assert!(matches!(err, IpaValidationError::NotFound(_)));

    // 2. Empty file
    let empty_path = temp.path().join("empty.ipa");
    File::create(&empty_path).unwrap();
    let err = IpaValidator::validate_ipa(&empty_path).unwrap_err();
    assert_eq!(err, IpaValidationError::EmptyFile);

    // 3. Fake non-zip file named .ipa
    let fake_path = temp.path().join("fake.ipa");
    std::fs::write(&fake_path, b"THIS IS NOT A ZIP ARCHIVE").unwrap();
    let err = IpaValidator::validate_ipa(&fake_path).unwrap_err();
    assert!(matches!(err, IpaValidationError::InvalidZipFormat(_)));

    // 4. Valid ZIP but missing Payload/ directory
    let no_payload_path = temp.path().join("no_payload.ipa");
    {
        let file = File::create(&no_payload_path).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file("some_folder/dummy.txt", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"hello").unwrap();
        zip.finish().unwrap();
    }
    let err = IpaValidator::validate_ipa(&no_payload_path).unwrap_err();
    assert_eq!(err, IpaValidationError::MissingPayloadDirectory);

    // 5. Valid ZIP with Payload/ but no .app bundle
    let no_app_path = temp.path().join("no_app.ipa");
    {
        let file = File::create(&no_app_path).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file("Payload/random.txt", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"hello").unwrap();
        zip.finish().unwrap();
    }
    let err = IpaValidator::validate_ipa(&no_app_path).unwrap_err();
    assert_eq!(err, IpaValidationError::MissingAppBundle);
}

#[test]
fn test_ipa_validator_accepts_valid_app_bundle() {
    let temp = tempdir().expect("Failed to create tempdir");
    let valid_ipa_path = temp.path().join("valid_fixture.ipa");

    let plist_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.ratana.fixture.fixtureApp</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
</dict>
</plist>"#;

    {
        let file = File::create(&valid_ipa_path).unwrap();
        let mut zip = ZipWriter::new(file);

        // Add Payload/Runner.app/Info.plist
        zip.start_file("Payload/Runner.app/Info.plist", SimpleFileOptions::default()).unwrap();
        zip.write_all(plist_xml.as_bytes()).unwrap();

        // Add dummy binary
        zip.start_file("Payload/Runner.app/Runner", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"MOCK_MACH_O_BINARY").unwrap();

        zip.finish().unwrap();
    }

    let report = IpaValidator::validate_ipa(&valid_ipa_path)
        .expect("Valid fixture IPA should pass structural validation");

    assert_eq!(report.app_name, Some("Runner.app".to_string()));
    assert_eq!(report.bundle_id, Some("com.ratana.fixture.fixtureApp".to_string()));
    assert_eq!(report.version, Some("1.0.0".to_string()));
    assert_eq!(report.status, IpaValidationStatus::PackageCreatedButNotFullyVerified);
}
