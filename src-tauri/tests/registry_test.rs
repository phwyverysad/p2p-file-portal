use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_register_and_retrieve_file_share() {
    let dir = tempdir().unwrap();
    let sample_file_path = dir.path().join("test_document.pdf");
    let mut f = File::create(&sample_file_path).unwrap();
    f.write_all(b"Hello World PDF Content").unwrap();

    let storage_path = dir.path().join("shares.json");
    let registry = local_file_sharer_lib::registry::FileRegistry::new(storage_path.clone());

    let item = registry
        .register(sample_file_path.to_str().unwrap(), Some("My Document".to_string()), None)
        .expect("should register file");

    assert!(item.id.starts_with("f_"));
    assert_eq!(item.original_name, "test_document.pdf");
    assert_eq!(item.title, "My Document");
    assert_eq!(item.size, 23);
    assert_eq!(item.downloads, 0);
    assert_eq!(item.views, 0);

    let retrieved = registry.get(&item.id).expect("should find share item");
    assert_eq!(retrieved.id, item.id);
    assert_eq!(retrieved.title, "My Document");

    // Test persistence: create a new registry instance pointing to the same storage_path
    let registry2 = local_file_sharer_lib::registry::FileRegistry::new(storage_path);
    let reloaded = registry2.get(&item.id).expect("should reload from disk");
    assert_eq!(reloaded.id, item.id);
}

#[test]
fn test_update_and_revoke_share() {
    let dir = tempdir().unwrap();
    let sample_file_path = dir.path().join("archive.zip");
    let mut f = File::create(&sample_file_path).unwrap();
    f.write_all(b"Zip binary content").unwrap();

    let storage_path = dir.path().join("shares.json");
    let registry = local_file_sharer_lib::registry::FileRegistry::new(storage_path);

    let item = registry
        .register(sample_file_path.to_str().unwrap(), None, None)
        .expect("should register file");

    assert_eq!(item.title, "archive.zip");

    let updated = registry
        .update_meta(&item.id, Some("Updated Title".to_string()), Some("My Desc".to_string()), Some(true))
        .expect("should update meta");
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.description, "My Desc");
    assert_eq!(updated.direct_download, true);

    registry.increment_views(&item.id);
    registry.increment_downloads(&item.id);

    let retrieved = registry.get(&item.id).unwrap();
    assert_eq!(retrieved.views, 1);
    assert_eq!(retrieved.downloads, 1);

    let revoked = registry.revoke(&item.id);
    assert!(revoked);
    assert!(registry.get(&item.id).is_none());

    // Test Undo / Restore:
    let restored = registry.restore(retrieved);
    assert_eq!(restored.id, item.id);
    assert_eq!(restored.views, 1);
    assert_eq!(restored.downloads, 1);
    assert_eq!(restored.direct_download, true);
    assert!(registry.get(&item.id).is_some());
}

#[test]
fn test_register_directory_share() {
    let dir = tempdir().unwrap();
    let folder_path = dir.path().join("my_shared_folder");
    std::fs::create_dir_all(&folder_path).unwrap();

    let f1 = folder_path.join("file1.txt");
    std::fs::write(&f1, b"12345").unwrap();

    let sub = folder_path.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let f2 = sub.join("file2.txt");
    std::fs::write(&f2, b"67890").unwrap();

    let storage_path = dir.path().join("shares.json");
    let registry = local_file_sharer_lib::registry::FileRegistry::new(storage_path);

    let item = registry
        .register(folder_path.to_str().unwrap(), None, None)
        .expect("should register directory");

    assert!(item.is_directory);
    assert_eq!(item.file_count, 2);
    assert_eq!(item.size, 10);
    assert_eq!(item.mime_type, "application/zip");
    assert_eq!(item.original_name, "my_shared_folder");
}

#[test]
fn test_history_tracking_and_deletion_status() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("document.pdf");
    std::fs::write(&file_path, b"Sample Document").unwrap();

    let storage_path = dir.path().join("shares.json");
    let registry = local_file_sharer_lib::registry::FileRegistry::new(storage_path);

    // 1. Register file -> recorded in active shares & history
    let item = registry
        .register(file_path.to_str().unwrap(), Some("Doc".to_string()), None)
        .expect("should register file");

    let history = registry.list_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].item.id, item.id);
    assert!(history[0].file_exists, "File exists on disk");
    assert!(history[0].is_active, "Item is currently active");

    // 2. Revoke share -> removed from active shares, but preserved in history
    let revoked = registry.revoke(&item.id);
    assert!(revoked);
    assert!(registry.get(&item.id).is_none());

    let history_after_revoke = registry.list_history();
    assert_eq!(history_after_revoke.len(), 1);
    assert_eq!(history_after_revoke[0].is_active, false);
    assert_eq!(history_after_revoke[0].file_exists, true);

    // 3. Restore from history while file still exists
    let restored = registry.restore_from_history(&item.id).expect("should restore from history");
    assert_eq!(restored.id, item.id);
    assert!(registry.get(&item.id).is_some());

    // 4. Revoke again and physically remove file from disk
    registry.revoke(&item.id);
    std::fs::remove_file(&file_path).unwrap();

    let history_missing = registry.list_history();
    assert_eq!(history_missing.len(), 1);
    assert_eq!(history_missing[0].file_exists, false, "Should detect file missing on disk");

    // 5. Attempting to restore missing file should return Err
    let restore_err = registry.restore_from_history(&item.id);
    assert!(restore_err.is_err(), "Cannot restore file deleted from disk");

    // 6. Delete item from history
    let deleted_count = registry.delete_history_items(&[item.id.clone()]);
    assert_eq!(deleted_count, 1);
    assert_eq!(registry.list_history().len(), 0);
}

