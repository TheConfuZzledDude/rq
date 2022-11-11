use tauri::{AppHandle, Manager};
use tauri_egui::{
    eframe,
    egui::{self, Vec2},
    EguiPluginHandle,
};

use crate::commands::new_queue;

pub struct NewQueueApp {
    name: String,
    restrict_to_group: String,
    app_handle: AppHandle,
}

impl NewQueueApp {}

impl eframe::App for NewQueueApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, move |ui| {
            let app = Box::new(self.app_handle.clone());

            ui.label("Name");
            let _name_field = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(&mut self.name),
            );

            ui.label("Restrict To Group");
            let _restrict_to_group_field = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(&mut self.restrict_to_group),
            );

            let name = self.name.clone();
            let restrict_to_group = self.restrict_to_group.clone();

            if ui.button("Submit").clicked() {
                frame.close();
                tauri::async_runtime::spawn(async move {
                    let app = app.clone();
                    new_queue(
                        app.state(),
                        &name,
                        if restrict_to_group.is_empty() {
                            None
                        } else {
                            Some(&restrict_to_group)
                        },
                    )
                    .await
                    .unwrap();
                });
            }
        });
    }
}

impl NewQueueApp {
    fn new(name: String, restrict_to_group: String, app_handle: AppHandle) -> Self {
        Self {
            name,
            restrict_to_group,
            app_handle,
        }
    }
    pub fn launch(app: AppHandle) {
        let egui_handle = app.state::<EguiPluginHandle>();

        let native_options = eframe::NativeOptions {
            resizable: false,
            initial_window_size: Some(Vec2::new(500.0, 400.0)),
            ..Default::default()
        };

        let app = app.clone();
        let _window = egui_handle
            .create_window(
                String::from("new_queue"),
                Box::new(move |_| {
                    Box::new(NewQueueApp::new(String::from(""), String::from(""), app))
                }),
                String::from("New Queue"),
                native_options,
            )
            .unwrap();
    }
}
