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
fn test_thumbnail_user_files() {
    let mc_path = "C:\\Users\\woran\\Downloads\\Pick Up Display v1.1.0.mcaddon";
    if std::path::Path::new(mc_path).exists() {
        let thumb = get_file_thumbnail(mc_path, 256);
        assert!(thumb.is_some(), "mcaddon icon should be extracted");
        let bytes = thumb.unwrap();
        assert_eq!(&bytes[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        let decoded = image::load_from_memory(&bytes).expect("should decode valid PNG");
        println!("mcaddon extracted size: {}x{}", decoded.width(), decoded.height());
        // Pure icon is 32x32 (not a 256x256 fake document plate with 200px empty margin)
        assert_eq!(decoded.width(), 32);
        assert_eq!(decoded.height(), 32);
    }

    let authme_path = "C:\\Users\\woran\\Downloads\\authme (3).exe";
    if std::path::Path::new(authme_path).exists() {
        let thumb = get_file_thumbnail(authme_path, 256);
        assert!(thumb.is_some(), "authme icon should be extracted");
        let bytes = thumb.unwrap();
        assert_eq!(&bytes[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        let decoded = image::load_from_memory(&bytes).expect("should decode valid PNG");
        println!("authme extracted size: {}x{}", decoded.width(), decoded.height());
        // Full resolution icon extracted via PrivateExtractIconsW
        assert!(decoded.width() >= 32);
        assert!(decoded.height() >= 32);
    }
}

#[test]
fn test_thumbnail_non_existent_file() {
    let thumb = get_file_thumbnail("C:\\non_existent_path_xyz_123.bin", 64);
    assert!(thumb.is_none(), "Non-existent file should return None");
}

