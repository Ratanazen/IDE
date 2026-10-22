use ratana_flutter_studio_lib::projects::{CreateProjectRequest, ProjectManager};

#[test]
fn test_project_name_validation() {
    assert!(ProjectManager::is_valid_project_name("my_app"));
    assert!(ProjectManager::is_valid_project_name("flutter_test_1"));
    assert!(ProjectManager::is_valid_project_name("app"));

    // Invalid names
    assert!(!ProjectManager::is_valid_project_name("123app"));
    assert!(!ProjectManager::is_valid_project_name("MyApp"));
    assert!(!ProjectManager::is_valid_project_name("my-app"));
    assert!(!ProjectManager::is_valid_project_name("my app"));
    assert!(!ProjectManager::is_valid_project_name(""));
}

#[tokio::test]
async fn test_create_import_analyze_fixture_project() {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let parent_path = temp_dir.path().to_path_buf();

    // 1. Create project using real flutter create
    let create_req = CreateProjectRequest {
        name: "test_fixture".to_string(),
        org: Some("com.ratana.fixture".to_string()),
        parent_dir: parent_path.clone(),
        platforms: vec!["web".to_string(), "linux".to_string(), "android".to_string()],
    };

    let meta = ProjectManager::create_project(create_req)
        .expect("flutter create should succeed for fixture project");

    assert_eq!(meta.name, "test_fixture");
    assert!(meta.path.exists());
    assert!(meta.path.join("pubspec.yaml").exists());

    // Verify platforms
    assert!(meta.platforms.contains(&"web".to_string()));
    assert!(meta.platforms.contains(&"linux".to_string()));
    assert!(meta.platforms.contains(&"android".to_string()));

    // 2. Import project
    let imported = ProjectManager::import_project(&meta.path)
        .expect("Importing created project should succeed");
    assert_eq!(imported.name, "test_fixture");
    assert!(imported.description.is_some());

    // 3. Run flutter pub get
    let pub_get_res = ProjectManager::run_project_action(&meta.path, "pub_get")
        .expect("flutter pub get action failed to run");
    assert!(pub_get_res.success, "flutter pub get should succeed: {}", pub_get_res.stderr);
    assert_eq!(pub_get_res.exit_code, Some(0));

    // 4. Run flutter analyze
    let analyze_res = ProjectManager::run_project_action(&meta.path, "analyze")
        .expect("flutter analyze action failed to run");
    assert!(analyze_res.success, "flutter analyze should succeed on fresh project: {}", analyze_res.stderr);
    assert_eq!(analyze_res.exit_code, Some(0));

    // 5. Run flutter test
    let test_res = ProjectManager::run_project_action(&meta.path, "test")
        .expect("flutter test action failed to run");
    assert!(test_res.success, "flutter test should succeed on default counter test: {}", test_res.stderr);
    assert_eq!(test_res.exit_code, Some(0));
}
