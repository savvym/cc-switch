use cc_switch_core::{AppType, Database, Provider};

#[test]
fn test_app_type_from_str() {
    assert_eq!(AppType::from_str("claude"), Some(AppType::Claude));
    assert_eq!(AppType::from_str("CLAUDE"), Some(AppType::Claude));
    assert_eq!(AppType::from_str("codex"), Some(AppType::Codex));
    assert_eq!(AppType::from_str("gemini"), Some(AppType::Gemini));
    assert_eq!(AppType::from_str("invalid"), None);
}

#[test]
fn test_app_type_as_str() {
    assert_eq!(AppType::Claude.as_str(), "claude");
    assert_eq!(AppType::Codex.as_str(), "codex");
    assert_eq!(AppType::Gemini.as_str(), "gemini");
}

#[test]
fn test_database_memory() {
    let db = Database::memory().expect("Failed to create in-memory database");

    // Should be empty initially
    let providers = db.get_all_providers("claude").expect("Failed to get providers");
    assert!(providers.is_empty());
}

#[test]
fn test_database_save_and_get_provider() {
    let db = Database::memory().expect("Failed to create in-memory database");

    let provider = Provider {
        id: "test-id".to_string(),
        name: "Test Provider".to_string(),
        settings_config: serde_json::json!({"key": "value"}),
        website_url: None,
        category: Some("custom".to_string()),
        created_at: Some(1234567890),
        sort_index: None,
        notes: None,
        meta: None,
        icon: None,
        icon_color: None,
        is_proxy_target: None,
    };

    db.save_provider("claude", &provider).expect("Failed to save provider");

    let retrieved = db.get_provider_by_id("test-id", "claude")
        .expect("Failed to get provider")
        .expect("Provider not found");

    assert_eq!(retrieved.id, "test-id");
    assert_eq!(retrieved.name, "Test Provider");
}

#[test]
fn test_database_delete_provider() {
    let db = Database::memory().expect("Failed to create in-memory database");

    let provider = Provider {
        id: "delete-test".to_string(),
        name: "To Delete".to_string(),
        settings_config: serde_json::json!({}),
        website_url: None,
        category: None,
        created_at: None,
        sort_index: None,
        notes: None,
        meta: None,
        icon: None,
        icon_color: None,
        is_proxy_target: None,
    };

    db.save_provider("claude", &provider).expect("Failed to save");
    db.delete_provider("claude", "delete-test").expect("Failed to delete");

    let result = db.get_provider_by_id("delete-test", "claude").expect("Query failed");
    assert!(result.is_none());
}

#[test]
fn test_database_set_current_provider() {
    let db = Database::memory().expect("Failed to create in-memory database");

    // Create two providers
    for id in ["provider-1", "provider-2"] {
        let provider = Provider {
            id: id.to_string(),
            name: format!("Provider {}", id),
            settings_config: serde_json::json!({}),
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            is_proxy_target: None,
        };
        db.save_provider("claude", &provider).expect("Failed to save");
    }

    // Set provider-1 as current
    db.set_current_provider("claude", "provider-1").expect("Failed to set current");
    let current = db.get_current_provider("claude").expect("Failed to get current");
    assert_eq!(current, Some("provider-1".to_string()));

    // Switch to provider-2
    db.set_current_provider("claude", "provider-2").expect("Failed to set current");
    let current = db.get_current_provider("claude").expect("Failed to get current");
    assert_eq!(current, Some("provider-2".to_string()));
}

#[test]
fn test_database_get_all_providers() {
    let db = Database::memory().expect("Failed to create in-memory database");

    // Add providers for different app types
    for (app, id) in [("claude", "claude-1"), ("codex", "codex-1")] {
        let provider = Provider {
            id: id.to_string(),
            name: format!("Provider {}", id),
            settings_config: serde_json::json!({}),
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            is_proxy_target: None,
        };
        db.save_provider(app, &provider).expect("Failed to save");
    }

    let claude_providers = db.get_all_providers("claude").expect("Failed to get");
    let codex_providers = db.get_all_providers("codex").expect("Failed to get");

    assert_eq!(claude_providers.len(), 1);
    assert_eq!(codex_providers.len(), 1);
    assert!(claude_providers.contains_key("claude-1"));
    assert!(codex_providers.contains_key("codex-1"));
}
