/// Check if any audio session is actively playing.
/// Uses Win32 COM IAudioSessionManager2.
pub fn is_audio_playing() -> bool {
    unsafe { is_audio_playing_inner().unwrap_or(false) }
}

unsafe fn is_audio_playing_inner() -> Option<bool> {
    use windows::Win32::Media::Audio::{
        AudioSessionStateActive, IAudioSessionControl, IAudioSessionEnumerator,
        IAudioSessionManager2, IMMDeviceEnumerator, MMDeviceEnumerator,
        eConsole, eRender,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    // Initialize COM for this thread (MTA is safe on tokio thread pool threads).
    // Ignore RPC_E_CHANGED_MODE / S_FALSE if already initialized.
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

    let enumerator: IMMDeviceEnumerator =
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;

    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;

    let session_manager: IAudioSessionManager2 =
        device.Activate(CLSCTX_ALL, None).ok()?;

    let session_enum: IAudioSessionEnumerator =
        session_manager.GetSessionEnumerator().ok()?;

    let count = session_enum.GetCount().ok()?;

    for i in 0..count {
        let session_ctrl: IAudioSessionControl = session_enum.GetSession(i).ok()?;
        if let Ok(state) = session_ctrl.GetState() {
            if state == AudioSessionStateActive {
                return Some(true);
            }
        }
    }

    Some(false)
}
