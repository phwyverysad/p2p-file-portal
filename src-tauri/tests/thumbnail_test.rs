use local_file_sharer_lib::windows::get_file_thumbnail;
use tempfile::tempdir;

#[test]
fn test_thumbnail_extraction_for_exe() {
    #[cfg(target_os = "windows")]
    {
        let explorer_path = "C:\\Windows\\explorer.exe";
        if std::path::Path::new(explorer_path).exists() {
            let thumb = get_file_thumbnail(explorer_path, 128);
            assert!(thumb.is_some(), "Explorer.exe icon should be extracted");
            let bytes = thumb.unwrap();
            assert!(!bytes.is_empty(), "Thumbnail bytes should not be empty");
            // Check PNG header: [137, 80, 78, 71, 13, 10, 26, 10]
            assert_eq!(&bytes[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
            let decoded = image::load_from_memory(&bytes).expect("should decode valid PNG");
            assert!(decoded.width() > 0 && decoded.height() > 0);
        }
    }
}

#[test]
fn test_thumbnail_extraction_for_image() {
    let dir = tempdir().unwrap();
    let img_path = dir.path().join("test_img.png");

    // Create a 16x16 red PNG image
    let img = image::RgbaImage::from_fn(16, 16, |_, _| image::Rgba([255, 0, 0, 255]));
    img.save(&img_path).unwrap();

    let thumb = get_file_thumbnail(img_path.to_str().unwrap(), 64);
    assert!(thumb.is_some(), "Image thumbnail should be extracted");
    let bytes = thumb.unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
}

#[test]
fn test_thumbnail_non_existent_file() {
    let thumb = get_file_thumbnail("C:\\non_existent_path_xyz_123.bin", 64);
    assert!(thumb.is_none(), "Non-existent file should return None");
}
