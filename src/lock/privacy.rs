use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SAM_FLAGS, REG_SZ, RegCloseKey,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::w;

const KEY_PATH: windows::core::PCWSTR =
    w!(r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone");
const VALUE_NAME: windows::core::PCWSTR = w!("Value");

pub const ALLOW: &str = "Allow";
pub const DENY: &str = "Deny";

fn to_wide_bytes(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn open_key(access: REG_SAM_FLAGS) -> windows::core::Result<HKEY> {
    let mut key = HKEY::default();
    let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, KEY_PATH, Some(0), access, &mut key) };
    if status == ERROR_SUCCESS {
        Ok(key)
    } else {
        Err(windows::core::Error::from(status))
    }
}

pub fn read() -> windows::core::Result<Option<String>> {
    let key = match open_key(KEY_QUERY_VALUE) {
        Ok(key) => key,
        Err(error) if error.code() == ERROR_FILE_NOT_FOUND.to_hresult() => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut kind = REG_SZ;
    let mut size: u32 = 0;
    let status = unsafe {
        RegQueryValueExW(key, VALUE_NAME, None, Some(&mut kind), None, Some(&mut size))
    };
    if status != ERROR_SUCCESS {
        unsafe { let _ = RegCloseKey(key); }
        return Err(windows::core::Error::from(status));
    }
    let mut buffer = vec![0u8; size as usize];
    let status = unsafe {
        RegQueryValueExW(
            key,
            VALUE_NAME,
            None,
            Some(&mut kind),
            Some(buffer.as_mut_ptr()),
            Some(&mut size),
        )
    };
    unsafe { let _ = RegCloseKey(key); }
    if status != ERROR_SUCCESS {
        return Err(windows::core::Error::from(status));
    }
    let wide: Vec<u16> = buffer
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|&unit| unit != 0)
        .collect();
    Ok(Some(String::from_utf16_lossy(&wide)))
}

pub fn write(value: &str) -> windows::core::Result<()> {
    let key = open_key(KEY_SET_VALUE)?;
    let bytes = to_wide_bytes(value);
    let status = unsafe { RegSetValueExW(key, VALUE_NAME, None, REG_SZ, Some(&bytes)) };
    unsafe { let _ = RegCloseKey(key); }
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(windows::core::Error::from(status))
    }
}
