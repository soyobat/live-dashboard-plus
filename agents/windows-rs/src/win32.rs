use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetSystemMetrics, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, SM_CXSCREEN, SM_CYSCREEN,
};
use windows::core::PWSTR;

/// Returns the current foreground window's (process_name, title), or None.
pub fn get_foreground_info() -> Option<(String, String)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return None;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, &mut buf);
        let title = String::from_utf16_lossy(&buf)
            .trim_end_matches('\0')
            .trim()
            .to_string();
        if title.is_empty() {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let proc_name = process_name_by_pid(pid).unwrap_or_else(|| "unknown".into());

        Some((proc_name, title))
    }
}

/// Get process executable name (e.g. "chrome.exe") from a PID.
pub fn process_name_by_pid(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; 260];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        ok.ok()?;
        let full_path = String::from_utf16_lossy(&buf[..size as usize]);
        let name = full_path
            .split('\\')
            .last()
            .unwrap_or(&full_path)
            .to_lowercase();
        Some(name)
    }
}

/// Returns seconds since last keyboard/mouse input.
pub fn get_idle_secs() -> f64 {
    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if !GetLastInputInfo(&mut lii).as_bool() {
            return 0.0;
        }
        let tick_now = windows::Win32::System::SystemInformation::GetTickCount64();
        let elapsed_ms = tick_now.wrapping_sub(lii.dwTime as u64);
        elapsed_ms as f64 / 1000.0
    }
}

/// Returns true if the foreground window fills the entire screen.
pub fn is_fullscreen() -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        rect.left <= 0 && rect.top <= 0 && rect.right >= sw && rect.bottom >= sh
    }
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percent: u8,
    pub charging: bool,
}

/// Returns battery info, or None if no battery is present.
pub fn get_battery() -> Option<BatteryInfo> {
    unsafe {
        let mut status = SYSTEM_POWER_STATUS::default();
        if GetSystemPowerStatus(&mut status).is_err() {
            return None;
        }
        // BatteryFlag == 128 means "no system battery"; 255 means unknown
        if status.BatteryFlag == 128 || status.BatteryFlag == 255 {
            return None;
        }
        if status.BatteryLifePercent == 255 {
            return None;
        }
        Some(BatteryInfo {
            percent: status.BatteryLifePercent,
            charging: status.ACLineStatus == 1,
        })
    }
}

/// Enumerate all visible top-level windows, calling `callback` with (hwnd, title, pid).
/// Stops enumeration if callback returns false.
pub fn enum_visible_windows<F>(mut callback: F)
where
    F: FnMut(HWND, String, u32) -> bool,
{
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let cb_ptr = lparam.0 as *mut &mut dyn FnMut(HWND, String, u32) -> bool;
        let cb = &mut **cb_ptr;

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return BOOL(1);
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, &mut buf);
        let title = String::from_utf16_lossy(&buf)
            .trim_end_matches('\0')
            .trim()
            .to_string();
        if title.is_empty() {
            return BOOL(1);
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        if cb(hwnd, title, pid) {
            BOOL(1)
        } else {
            BOOL(0)
        }
    }

    let cb_dyn: &mut dyn FnMut(HWND, String, u32) -> bool = &mut callback;
    let cb_ptr = &cb_dyn as *const _ as isize;
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(enum_proc),
            LPARAM(cb_ptr),
        )
        .ok();
    }
}

/// Show a native Windows message box.
pub fn message_box(title: &str, message: &str, error: bool) {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION};
    use windows::core::PCWSTR;

    let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let msg_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let flags = if error { MB_ICONERROR } else { MB_ICONINFORMATION };
    unsafe {
        MessageBoxW(
            HWND(std::ptr::null_mut()),
            PCWSTR(msg_w.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            flags,
        );
    }
}
