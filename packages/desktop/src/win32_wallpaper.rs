#[cfg(target_os = "windows")]
pub mod windows_wallpaper {
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, SMTO_NORMAL, SendMessageTimeoutW, SetParent,
    };
    use windows::core::{PCWSTR, w};

    static mut WORKERW: HWND = HWND(std::ptr::null_mut());

    unsafe extern "system" fn enum_windows_proc(
        tophandle: HWND,
        _lparam: LPARAM,
    ) -> windows::core::BOOL {
        let p = FindWindowExW(
            Some(tophandle),
            None,
            w!("SHELLDLL_DefView"),
            PCWSTR::null(),
        );
        if p.is_ok() {
            let workerw = FindWindowExW(None, Some(tophandle), w!("WorkerW"), PCWSTR::null());
            if let Ok(w) = workerw {
                WORKERW = w;
            }
        }
        windows::core::BOOL::from(true)
    }

    pub fn attach_to_desktop(hwnd: isize) {
        unsafe {
            let progman = FindWindowW(w!("Progman"), PCWSTR::null());
            if let Ok(progman_hwnd) = progman {
                let mut result: usize = 0;
                let _ = SendMessageTimeoutW(
                    progman_hwnd,
                    0x052C,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_NORMAL,
                    1000,
                    Some(&mut result),
                );

                let _ = EnumWindows(Some(enum_windows_proc), LPARAM(0));

                if !WORKERW.0.is_null() {
                    let _ = SetParent(HWND(hwnd as _), Some(WORKERW));
                }
            }
        }
    }
}
