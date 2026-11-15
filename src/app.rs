use std::sync::mpsc::{Receiver, Sender};

use eframe::egui::{self, Color32, RichText, Sense, Vec2};

use crate::icon;
use crate::theme;
use crate::worker::{self, Command, Event, Snapshot};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Active,
    Locking,
    Locked,
    Unlocking,
}

pub struct AppState {
    pub phase: Phase,
    pub error: Option<String>,
    snapshot: Option<Snapshot>,
    commands: Sender<Command>,
    events: Receiver<Event>,
}

impl AppState {
    pub fn new(initially_locked: bool) -> Self {
        let (commands, events) = worker::start();
        Self {
            phase: if initially_locked { Phase::Locked } else { Phase::Active },
            error: None,
            snapshot: None,
            commands,
            events,
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.phase, Phase::Locking | Phase::Unlocking)
    }

    pub fn poll(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Locked(report, snapshot) => {
                    self.phase = Phase::Locked;
                    self.snapshot = Some(snapshot);
                    self.error = failure_text(&report);
                }
                Event::Unlocked(report) => {
                    self.phase = Phase::Active;
                    self.snapshot = None;
                    self.error = failure_text(&report);
                }
                Event::Failed(message) => {
                    self.error = Some(message);
                    self.phase = match self.phase {
                        Phase::Locking => Phase::Active,
                        Phase::Unlocking => Phase::Locked,
                        other => other,
                    };
                }
            }
        }
    }

    fn toggle(&mut self) {
        self.error = None;
        match self.phase {
            Phase::Active => {
                self.phase = Phase::Locking;
                let _ = self.commands.send(Command::Lock);
            }
            Phase::Locked => {
                self.phase = Phase::Unlocking;
                let _ = self.commands.send(Command::Unlock);
            }
            _ => {}
        }
    }

    fn status(&self) -> (&'static str, &'static str, Color32) {
        match self.phase {
            Phase::Active => ("Микрофон активен", "Нажмите, чтобы заблокировать", theme::ACTIVE),
            Phase::Locking => ("Блокировка…", "Применяем защиту", theme::MUTED),
            Phase::Locked => ("Микрофон заблокирован", "Нажмите, чтобы вернуть доступ", theme::LOCKED),
            Phase::Unlocking => ("Возвращаем доступ…", "Восстанавливаем настройки", theme::MUTED),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let (title, hint, color) = self.status();
        ui.vertical_centered(|ui| {
            ui.add_space(14.0);
            ui.label(
                RichText::new(title)
                    .family(egui::FontFamily::Name("semibold".into()))
                    .size(24.0)
                    .color(color),
            );
            ui.label(RichText::new(hint).color(theme::MUTED));
            ui.add_space(20.0);
            self.mic_button(ui);
            ui.add_space(22.0);
            if let Some(error) = &self.error {
                ui.label(RichText::new(error).color(theme::ERROR));
            }
        });
    }

    fn mic_button(&mut self, ui: &mut egui::Ui) {
        let diameter = 168.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::click());
        let busy = self.is_busy();
        let label = if matches!(self.phase, Phase::Active | Phase::Locked) {
            match self.phase {
                Phase::Active => "Заблокировать микрофон",
                _ => "Вернуть доступ к микрофону",
            }
        } else {
            "Микрофон обрабатывается"
        };
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
        if !busy {
            response.clone().on_hover_cursor(egui::CursorIcon::PointingHand);
        }

        let locked = matches!(self.phase, Phase::Locking | Phase::Locked);
        let t = ui
            .ctx()
            .animate_bool_with_time(ui.scope_id().with("mic_state"), locked, 0.35);
        let slash = ui
            .ctx()
            .animate_bool_with_time(ui.scope_id().with("mic_slash"), locked, 0.45);
        let hover = ui
            .ctx()
            .animate_bool_with_time(ui.scope_id().with("mic_hover"), response.hovered() && !busy, 0.15);

        let base = theme::lerp_color(theme::ACTIVE, theme::LOCKED, t);
        let hover_color = theme::lerp_color(theme::ACTIVE_HOVER, theme::LOCKED_HOVER, t);
        let fill = theme::lerp_color(base, hover_color, hover);

        let painter = ui.painter_at(rect.expand(10.0));
        let center = rect.center();
        painter.circle_filled(center, diameter * 0.5, fill);
        icon::mic_icon(&painter, center, diameter * 0.5, theme::TEXT, fill, slash);

        if response.clicked() && !busy {
            self.toggle();
        }
    }
}

fn failure_text(report: &worker::Report) -> Option<String> {
    let mut parts = Vec::new();
    if report.privacy == Some(worker::MethodStatus::Failed) {
        parts.push(format!("реестр: {}", report.privacy_detail));
    }
    if report.core_audio == Some(worker::MethodStatus::Failed) {
        parts.push(format!("звук: {}", report.core_audio_detail));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}
