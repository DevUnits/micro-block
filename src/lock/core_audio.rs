use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};

#[derive(Clone, Debug)]
pub struct DeviceState {
    pub id: String,
    pub muted: bool,
    pub volume: f32,
}

pub struct ComGuard;

impl ComGuard {
    pub fn new() -> windows::core::Result<Self> {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_err() {
            return Err(windows::core::Error::from(hr));
        }
        Ok(Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

fn enumerator() -> windows::core::Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
}

fn endpoint(device: &IMMDevice) -> windows::core::Result<IAudioEndpointVolume> {
    unsafe { device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) }
}

fn device_id(device: &IMMDevice) -> String {
    match unsafe { device.GetId() } {
        Ok(wide) => unsafe { wide.to_string() }.unwrap_or_default(),
        Err(_) => String::new(),
    }
}

fn capture_devices() -> windows::core::Result<Vec<IMMDevice>> {
    let enumerator = enumerator()?;
    let collection = unsafe { enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE) }?;
    let count = unsafe { collection.GetCount() }?;
    let mut devices = Vec::with_capacity(count as usize);
    for index in 0..count {
        devices.push(unsafe { collection.Item(index) }?);
    }
    Ok(devices)
}

pub fn snapshot() -> windows::core::Result<Vec<DeviceState>> {
    let mut states = Vec::new();
    for device in capture_devices()? {
        let volume = endpoint(&device)?;
        let muted = unsafe { volume.GetMute() }?.as_bool();
        let level = unsafe { volume.GetMasterVolumeLevelScalar() }.unwrap_or(1.0);
        states.push(DeviceState {
            id: device_id(&device),
            muted,
            volume: level,
        });
    }
    Ok(states)
}

pub fn set_muted(muted: bool) -> windows::core::Result<usize> {
    let mut applied = 0;
    for device in capture_devices()? {
        let volume = endpoint(&device)?;
        unsafe { volume.SetMute(muted, std::ptr::null())? };
        if !muted && unsafe { volume.GetMasterVolumeLevelScalar() }.unwrap_or(1.0) < 0.01 {
            unsafe { volume.SetMasterVolumeLevelScalar(1.0, std::ptr::null())? };
        }
        applied += 1;
    }
    Ok(applied)
}

pub fn restore(states: &[DeviceState]) -> windows::core::Result<usize> {
    let mut restored = 0;
    for device in capture_devices()? {
        let id = device_id(&device);
        let Some(saved) = states.iter().find(|state| state.id == id) else {
            continue;
        };
        let volume = endpoint(&device)?;
        unsafe {
            volume.SetMute(saved.muted, std::ptr::null())?;
            volume.SetMasterVolumeLevelScalar(saved.volume, std::ptr::null())?;
        }
        restored += 1;
    }
    Ok(restored)
}
