use eframe::{egui, NativeOptions};
use eframe::egui::CtxRef;
use eframe::epi::Frame;

pub struct DemoApp {
    label: String,
    value: i32,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            label: "Hello World".to_owned(),
            value: 0,
        }
    }
}

impl epi::App for DemoApp {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
               ui.label(&self.label)
            });
    }

    fn name(&self) -> &str {
        "Foo"
    }
}

fn main() {
    eframe::run_native(Box::new(DemoApp::default()), NativeOptions::default())
}
