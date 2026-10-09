#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub fn register_context_menu(exe_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let clean_path = exe_path.strip_prefix(r"\\?\").unwrap_or(exe_path);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let target_paths = [
            r"Software\Classes\*\shell",
            r"Software\Classes\Directory\shell",
        ];

        for shell_path in target_paths {
            let (shell_key, _) = hkcu
                .create_subkey(shell_path)
                .map_err(|e| format!("Failed to open shell key {}: {}", shell_path, e))?;

            let (menu_key, _) = shell_key
                .create_subkey("FileSharer")
                .map_err(|e| format!("Failed to create FileSharer key: {}", e))?;

            menu_key
                .set_value("", &"แชร์ลิ้ง")
                .map_err(|e| format!("Failed to set menu label: {}", e))?;

            menu_key
                .set_value("Icon", &clean_path)
                .map_err(|e| format!("Failed to set menu icon: {}", e))?;

            let _ = menu_key.set_value("Position", &"Top");

            let (cmd_key, _) = menu_key
                .create_subkey("command")
                .map_err(|e| format!("Failed to create command key: {}", e))?;

            let cmd_string = format!("\"{}\" \"%1\"", clean_path);
            cmd_key
                .set_value("", &cmd_string)
                .map_err(|e| format!("Failed to set command value: {}", e))?;
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = exe_path;
        Ok(())
    }
}

pub fn unregister_context_menu() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let target_paths = [
            r"Software\Classes\*\shell",
            r"Software\Classes\Directory\shell",
        ];

        for shell_path in target_paths {
            if let Ok(shell_key) = hkcu.open_subkey_with_flags(shell_path, KEY_WRITE) {
                if let Ok(menu_key) = shell_key.open_subkey_with_flags("FileSharer", KEY_WRITE) {
                    let _ = menu_key.delete_subkey("command");
                }
                let _ = shell_key.delete_subkey("FileSharer");
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

pub fn is_context_menu_registered() -> bool {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let cmd_path = r"Software\Classes\*\shell\FileSharer\command";
        hkcu.open_subkey(cmd_path).is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_clipboard_text(text: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        extern "system" {
            fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
            fn CloseClipboard() -> i32;
            fn EmptyClipboard() -> i32;
            fn SetClipboardData(uFormat: u32, hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut std::ffi::c_void;
            fn GlobalLock(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn GlobalUnlock(hMem: *mut std::ffi::c_void) -> i32;
        }

        const CF_UNICODETEXT: u32 = 13;
        const GMEM_MOVEABLE: u32 = 0x0002;

        let wide: Vec<u16> = OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * 2;

        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Err("Failed to open Windows clipboard".to_string());
            }
            EmptyClipboard();
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if h_mem.is_null() {
                CloseClipboard();
                return Err("Failed to allocate global memory".to_string());
            }
            let ptr = GlobalLock(h_mem) as *mut u16;
            if ptr.is_null() {
                CloseClipboard();
                return Err("Failed to lock global memory".to_string());
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            GlobalUnlock(h_mem);
            SetClipboardData(CF_UNICODETEXT, h_mem);
            CloseClipboard();
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = text;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case, non_camel_case_types, dead_code)]
mod thumbnail {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use image::ImageEncoder;

    #[repr(C)]
    struct SIZE {
        cx: i32,
        cy: i32,
    }

    #[repr(C)]
    struct BITMAP {
        bmType: i32,
        bmWidth: i32,
        bmHeight: i32,
        bmWidthBytes: i32,
        bmPlanes: u16,
        bmBitsPixel: u16,
        bmBits: *mut std::ffi::c_void,
    }

    #[repr(C)]
    struct BITMAPINFOHEADER {
        biSize: u32,
        biWidth: i32,
        biHeight: i32,
        biPlanes: u16,
        biBitCount: u16,
        biCompression: u32,
        biSizeImage: u32,
        biXPelsPerMeter: i32,
        biYPelsPerMeter: i32,
        biClrUsed: u32,
        biClrImportant: u32,
    }

    #[repr(C)]
    struct BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER,
        bmiColors: [u32; 1],
    }

    #[repr(C)]
    struct IShellItemImageFactoryVtbl {
        pub QueryInterface: unsafe extern "system" fn(this: *mut std::ffi::c_void, riid: *const std::ffi::c_void, ppv: *mut *mut std::ffi::c_void) -> i32,
        pub AddRef: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
        pub Release: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
        pub GetImage: unsafe extern "system" fn(this: *mut std::ffi::c_void, size: SIZE, flags: u32, phbm: *mut *mut std::ffi::c_void) -> i32,
    }

    #[repr(C)]
    struct IShellItemImageFactory {
        pub lpVtbl: *const IShellItemImageFactoryVtbl,
    }

    #[repr(C)]
    struct SHFILEINFOW {
        hIcon: *mut std::ffi::c_void,
        iIcon: i32,
        dwAttributes: u32,
        szDisplayName: [u16; 260],
        szTypeName: [u16; 80],
    }

    #[repr(C)]
    struct ICONINFO {
        fIcon: i32,
        xHotspot: u32,
        yHotspot: u32,
        hbmMask: *mut std::ffi::c_void,
        hbmColor: *mut std::ffi::c_void,
    }

    extern "system" {
        fn SHCreateItemFromParsingName(
            pszPath: *const u16,
            pbc: *mut std::ffi::c_void,
            riid: *const u8,
            ppv: *mut *mut std::ffi::c_void,
        ) -> i32;
        fn CoInitializeEx(pvReserved: *mut std::ffi::c_void, dwCoInit: u32) -> i32;
        fn DeleteObject(ho: *mut std::ffi::c_void) -> i32;
        fn GetDC(hWnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn ReleaseDC(hWnd: *mut std::ffi::c_void, hDC: *mut std::ffi::c_void) -> i32;
        fn GetObjectW(h: *mut std::ffi::c_void, c: i32, pv: *mut std::ffi::c_void) -> i32;
        fn GetDIBits(
            hdc: *mut std::ffi::c_void,
            hbm: *mut std::ffi::c_void,
            start: u32,
            cLines: u32,
            lpvBits: *mut std::ffi::c_void,
            lpbmi: *mut BITMAPINFO,
            usage: u32,
        ) -> i32;
        fn SHGetFileInfoW(
            pszPath: *const u16,
            dwFileAttributes: u32,
            psfi: *mut SHFILEINFOW,
            cbFileInfo: u32,
            uFlags: u32,
        ) -> usize;
        fn DestroyIcon(hIcon: *mut std::ffi::c_void) -> i32;
        fn GetIconInfo(hIcon: *mut std::ffi::c_void, piconinfo: *mut ICONINFO) -> i32;
    }

    const SIIGBF_BIGGERSIZEOK: u32 = 0x01;
    const SHGFI_ICON: u32 = 0x000000100;
    const SHGFI_LARGEICON: u32 = 0x000000000;
    const DIB_RGB_COLORS: u32 = 0;

    const IID_ISHELLITEMIMAGEFACTORY: [u8; 16] = [
        0x79, 0x8b, 0xc1, 0xbc, 0x16, 0xba, 0x2f, 0x44, 0x80, 0xc4, 0x8a, 0x59, 0xc3, 0x0c, 0x46, 0x3b,
    ];

    unsafe fn hbitmap_to_png(hbm: *mut std::ffi::c_void) -> Option<Vec<u8>> {
        if hbm.is_null() {
            return None;
        }
        let mut bmp: BITMAP = std::mem::zeroed();
        if GetObjectW(hbm, std::mem::size_of::<BITMAP>() as i32, &mut bmp as *mut _ as *mut _) == 0 {
            return None;
        }

        let width = bmp.bmWidth;
        let height = bmp.bmHeight;
        if width <= 0 || height <= 0 {
            return None;
        }

        let hdc = GetDC(ptr::null_mut());
        if hdc.is_null() {
            return None;
        }

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width;
        bmi.bmiHeader.biHeight = -height;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = 0;

        let num_pixels = (width * height) as usize;
        let mut buf = vec![0u8; num_pixels * 4];

        let lines = GetDIBits(
            hdc,
            hbm,
            0,
            height as u32,
            buf.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );
        ReleaseDC(ptr::null_mut(), hdc);

        if lines == 0 {
            return None;
        }

        let mut has_non_zero_alpha = false;
        for pixel in buf.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            if pixel[3] != 0 {
                has_non_zero_alpha = true;
            }
        }

        if !has_non_zero_alpha {
            for pixel in buf.chunks_exact_mut(4) {
                pixel[3] = 255;
            }
        }

        let mut png_bytes = Vec::new();
        let res = image::codecs::png::PngEncoder::new(&mut png_bytes).write_image(
            &buf,
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        );

        if res.is_ok() {
            Some(png_bytes)
        } else {
            None
        }
    }

    unsafe fn hicon_to_png(hicon: *mut std::ffi::c_void) -> Option<Vec<u8>> {
        if hicon.is_null() {
            return None;
        }
        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(hicon, &mut icon_info) == 0 {
            return None;
        }
        let res = hbitmap_to_png(icon_info.hbmColor);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask);
        }
        res
    }

    pub fn get_thumbnail(path: &str, target_size: u32) -> Option<Vec<u8>> {
        let clean_path = path.strip_prefix(r"\\?\").unwrap_or(path);
        let path_obj = std::path::Path::new(clean_path);
        if !path_obj.exists() {
            return None;
        }

        let wide: Vec<u16> = OsStr::new(clean_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let _ = CoInitializeEx(ptr::null_mut(), 0x02);

            let mut factory_ptr: *mut std::ffi::c_void = ptr::null_mut();
            let hr = SHCreateItemFromParsingName(
                wide.as_ptr(),
                ptr::null_mut(),
                IID_ISHELLITEMIMAGEFACTORY.as_ptr(),
                &mut factory_ptr,
            );

            if hr == 0 && !factory_ptr.is_null() {
                let factory = factory_ptr as *mut IShellItemImageFactory;
                let mut hbm: *mut std::ffi::c_void = ptr::null_mut();
                let size = SIZE {
                    cx: target_size as i32,
                    cy: target_size as i32,
                };
                let get_img_hr = ((*(*factory).lpVtbl).GetImage)(factory_ptr, size, SIIGBF_BIGGERSIZEOK, &mut hbm);
                ((*(*factory).lpVtbl).Release)(factory_ptr);

                if get_img_hr == 0 && !hbm.is_null() {
                    let png = hbitmap_to_png(hbm);
                    DeleteObject(hbm);
                    if png.is_some() {
                        return png;
                    }
                }
            }

            let mut shfi: SHFILEINFOW = std::mem::zeroed();
            let ret = SHGetFileInfoW(
                wide.as_ptr(),
                0,
                &mut shfi,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );

            if ret != 0 && !shfi.hIcon.is_null() {
                let png = hicon_to_png(shfi.hIcon);
                DestroyIcon(shfi.hIcon);
                if png.is_some() {
                    return png;
                }
            }
        }

        if let Ok(img) = image::open(clean_path) {
            let thumb = img.thumbnail(target_size, target_size);
            let mut png_bytes = Vec::new();
            if thumb.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).is_ok() {
                return Some(png_bytes);
            }
        }

        None
    }
}

pub fn get_file_thumbnail(path: &str, target_size: u32) -> Option<Vec<u8>> {
    #[cfg(target_os = "windows")]
    {
        thumbnail::get_thumbnail(path, target_size)
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(img) = image::open(path) {
            let thumb = img.thumbnail(target_size, target_size);
            let mut png_bytes = Vec::new();
            if thumb.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).is_ok() {
                return Some(png_bytes);
            }
        }
        None
    }
}

