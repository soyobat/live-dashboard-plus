use std::process::Command;
use tracing::{info, warn};
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    REG_SZ, REG_VALUE_TYPE,
};

const AUTOSTART_NAME: &str = "LiveDashboardAgent";
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn get_autostart_command() -> String {
    let exe = std::env::current_exe()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    format!("\"{}\"", exe)
}

fn has_registry_autostart() -> bool {
    let key_w = to_wide(RUN_KEY);
    let name_w = to_wide(AUTOSTART_NAME);
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_w.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return false;
        }
        let mut reg_type = REG_VALUE_TYPE::default();
        let mut size = 0u32;
        let found = RegQueryValueExW(
            hkey,
            PCWSTR(name_w.as_ptr()),
            None,
            Some(&mut reg_type),
            None,
            Some(&mut size),
        )
        .is_ok()
            && size > 0;
        let _ = RegCloseKey(hkey);
        found
    }
}

fn set_registry_autostart(enabled: bool) -> bool {
    let key_w = to_wide(RUN_KEY);
    let name_w = to_wide(AUTOSTART_NAME);
    unsafe {
        let mut hkey = HKEY::default();
        if RegCreateKeyW(
            HKEY_CURRENT_USER,
            PCWSTR(key_w.as_ptr()),
            &mut hkey,
        )
        .is_err()
        {
            warn!("无法打开注册表 Run key");
            return false;
        }

        let ok = if enabled {
            let cmd = get_autostart_command();
            let val_w = to_wide(&cmd);
            let bytes = std::slice::from_raw_parts(
                val_w.as_ptr() as *const u8,
                val_w.len() * 2,
            );
            RegSetValueExW(
                hkey,
                PCWSTR(name_w.as_ptr()),
                0,
                REG_SZ,
                Some(bytes),
            )
            .is_ok()
        } else {
            let _ = RegDeleteValueW(hkey, PCWSTR(name_w.as_ptr()));
            true
        };

        let _ = RegCloseKey(hkey);
        ok
    }
}

fn has_legacy_task() -> bool {
    Command::new("schtasks")
        .args(["/query", "/tn", AUTOSTART_NAME])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn remove_legacy_task() -> bool {
    if !has_legacy_task() {
        return true;
    }
    match Command::new("schtasks")
        .args(["/delete", "/tn", AUTOSTART_NAME, "/f"])
        .output()
    {
        Ok(o) if o.status.success() => true,
        Ok(o) => {
            let msg = String::from_utf8_lossy(&o.stderr);
            warn!("删除旧计划任务失败: {msg}");
            false
        }
        Err(e) => {
            warn!("删除旧计划任务失败: {e}");
            false
        }
    }
}

/// Check if the agent is set to run at Windows logon.
pub fn is_enabled() -> bool {
    has_registry_autostart() || has_legacy_task()
}

/// Enable or disable autostart. Returns true on success.
pub fn set_enabled(enabled: bool) -> bool {
    if !enabled {
        let r = set_registry_autostart(false);
        let l = remove_legacy_task();
        if r && l {
            info!("开机自启已关闭");
        }
        r && l
    } else {
        if set_registry_autostart(true) {
            let _ = remove_legacy_task();
            info!("开机自启已开启");
            true
        } else {
            false
        }
    }
}
