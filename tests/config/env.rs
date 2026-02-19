use api::config::env;

#[tokio::test]
async fn test_env() {
    let config = env::init_env().expect("Failed to initialize environment");
    assert!(!config.api_url.is_empty(), "API URL should not be empty");
    assert!(
        !config.database_url.is_empty(),
        "Database URL should not be empty"
    );
    assert!(
        !config.jwt_secret.is_empty(),
        "JWT secret should not be empty"
    );
}
