#![windows_subsystem = "windows"]

mod app;
mod footer;
mod icon;
mod lock;
mod log;
mod theme;
mod worker;

use std::time::Duration;

use eframe::egui;

struct MicroBlockApp {
    state: app::AppState,
    styled: bool,
}

impl eframe::App for MicroBlockApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.styled {
            theme::install_fonts(ui.ctx());
            self.styled = true;
        }
        theme::apply(ui.ctx());
        self.state.poll();
        if self.state.is_busy() {
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
        egui::Panel::bottom("footer")
            .show_separator_line(false)
            .show(ui, |ui| {
                ui.add_space(8.0);
                footer::show(ui);
                ui.add_space(4.0);
            });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(12.0);
            self.state.show(ui);
        });
    }
}

fn load_icon() -> Option<egui::IconData> {
    let rgba = include_bytes!("../assets/icon-64.rgba").to_vec();
    Some(egui::IconData { rgba, width: 64, height: 64 })
}

fn main() -> eframe::Result {
    log::write("старт");
    if std::env::args().any(|arg| arg == "--unmute-all") {
        match worker::unmute_all() {
            Ok(count) => {
                log::write(&format!("unmute-all: {count}"));
                println!("устройств записи включено: {count}");
            }
            Err(error) => {
                log::write(&format!("unmute-all ошибка: {error}"));
                eprintln!("ошибка: {error}");
            }
        }
        return Ok(());
    }
    let initially_locked = worker::detect_locked();
    log::write(&format!("определено состояние: locked={initially_locked}"));
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Micro Block")
        .with_inner_size([400.0, 560.0])
        .with_resizable(false);
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Micro Block",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(MicroBlockApp {
                state: app::AppState::new(initially_locked),
                styled: false,
            }))
        }),
    )
}
