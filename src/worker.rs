use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

use crate::lock::{core_audio, privacy};

const SETTLE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MethodStatus {
    Ok,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub privacy: Option<MethodStatus>,
    pub core_audio: Option<MethodStatus>,
    pub privacy_detail: String,
    pub core_audio_detail: String,
}

impl Report {
    fn empty() -> Self {
        Self {
            privacy: None,
            core_audio: None,
            privacy_detail: String::new(),
            core_audio_detail: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub privacy_value: Option<String>,
    pub devices: Vec<core_audio::DeviceState>,
}

impl Snapshot {
    fn path() -> Option<PathBuf> {
        let base = std::env::var_os("LOCALAPPDATA")?;
        Some(PathBuf::from(base).join("MicroBlock").join("state.txt"))
    }

    fn save(&self) {
        let Some(path) = Self::path() else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut text = String::new();
        text.push_str(&format!(
            "privacy={}\n",
            self.privacy_value.clone().unwrap_or_default()
        ));
        for device in &self.devices {
            text.push_str(&format!("device={}|{}|{}\n", device.id, device.muted, device.volume));
        }
        let _ = std::fs::write(path, text);
    }

    fn load() -> Option<Self> {
        let text = std::fs::read_to_string(Self::path()?).ok()?;
        let mut snapshot = Self::default();
        for line in text.lines() {
            if let Some(value) = line.strip_prefix("privacy=") {
                snapshot.privacy_value = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            } else if let Some(rest) = line.strip_prefix("device=") {
                let mut parts = rest.splitn(3, '|');
                if let (Some(id), Some(muted), Some(volume)) = (parts.next(), parts.next(), parts.next()) {
                    snapshot.devices.push(core_audio::DeviceState {
                        id: id.to_string(),
                        muted: muted == "true",
                        volume: volume.parse().unwrap_or(1.0),
                    });
                }
            }
        }
        Some(snapshot)
    }

    fn clear() {
        if let Some(path) = Self::path() {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub enum Command {
    Lock,
    Unlock,
}

pub enum Event {
    Locked(Report, Snapshot),
    Unlocked(Report),
    Failed(String),
}

pub fn start() -> (Sender<Command>, Receiver<Event>) {
    let (command_tx, command_rx) = channel::<Command>();
    let (event_tx, event_rx) = channel::<Event>();
    thread::spawn(move || run(command_rx, event_tx));
    (command_tx, event_rx)
}

fn run(commands: Receiver<Command>, events: Sender<Event>) {
    let _com = match core_audio::ComGuard::new() {
        Ok(guard) => guard,
        Err(error) => {
            let _ = events.send(Event::Failed(format!("COM не инициализирован: {error}")));
            return;
        }
    };

    while let Ok(command) = commands.recv() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match command {
            Command::Lock => {
                let (report, snapshot) = lock();
                let _ = events.send(Event::Locked(report, snapshot));
            }
            Command::Unlock => {
                let report = unlock();
                let _ = events.send(Event::Unlocked(report));
            }
        }));
        if result.is_err() {
            let _ = events.send(Event::Failed("внутренняя ошибка".to_string()));
        }
    }
}

fn lock() -> (Report, Snapshot) {
    let mut report = Report::empty();
    let mut snapshot = Snapshot::default();

    match privacy::read() {
        Ok(value) => snapshot.privacy_value = value,
        Err(error) => {
            report.privacy = Some(MethodStatus::Failed);
            report.privacy_detail = error.message();
        }
    }
    match core_audio::snapshot() {
        Ok(states) => snapshot.devices = states,
        Err(error) => {
            report.core_audio = Some(MethodStatus::Failed);
            report.core_audio_detail = error.message();
        }
    }
    snapshot.save();

    if report.privacy.is_none() {
        match privacy::write(privacy::DENY) {
            Ok(()) => report.privacy = Some(MethodStatus::Ok),
            Err(error) => {
                report.privacy = Some(MethodStatus::Failed);
                report.privacy_detail = error.message();
            }
        }
    }
    if report.core_audio.is_none() {
        match core_audio::set_muted(true) {
            Ok(count) => {
                report.core_audio = Some(MethodStatus::Ok);
                report.core_audio_detail = format!("{count} устройств(а)");
            }
            Err(error) => {
                report.core_audio = Some(MethodStatus::Failed);
                report.core_audio_detail = error.message();
            }
        }
    }

    crate::log::write(&format!(
        "блокировка: реестр={:?} ({}), звук={:?} ({})",
        report.privacy, report.privacy_detail, report.core_audio, report.core_audio_detail
    ));
    (report, snapshot)
}

fn unlock() -> Report {
    let mut report = Report::empty();
    let snapshot = Snapshot::load();

    let target = snapshot
        .as_ref()
        .and_then(|snap| snap.privacy_value.clone())
        .unwrap_or_else(|| privacy::ALLOW.to_string());
    match privacy::write(&target) {
        Ok(()) => report.privacy = Some(MethodStatus::Ok),
        Err(error) => {
            report.privacy = Some(MethodStatus::Failed);
            report.privacy_detail = error.message();
        }
    }

    let result = match snapshot.as_ref().filter(|snap| !snap.devices.is_empty()) {
        Some(snap) => core_audio::restore(&snap.devices),
        None => Ok(0),
    };
    match result {
        Ok(count) => {
            report.core_audio = Some(MethodStatus::Ok);
            report.core_audio_detail = format!("{count} устройств(а)");
        }
        Err(error) => {
            report.core_audio = Some(MethodStatus::Failed);
            report.core_audio_detail = error.message();
        }
    }

    thread::sleep(SETTLE);
    Snapshot::clear();
    crate::log::write(&format!(
        "разблокировка: реестр={:?} ({}), звук={:?} ({})",
        report.privacy, report.privacy_detail, report.core_audio, report.core_audio_detail
    ));
    report
}

pub fn detect_locked() -> bool {
    privacy::read()
        .map(|value| value.as_deref() == Some(privacy::DENY))
        .unwrap_or(false)
}

pub fn unmute_all() -> Result<usize, String> {
    let _com = core_audio::ComGuard::new().map_err(|error| error.message())?;
    core_audio::set_muted(false).map_err(|error| error.message())
}
