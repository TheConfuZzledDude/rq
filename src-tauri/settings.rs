use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_egui::{
    eframe,
    egui::{self, FontId, RichText, Vec2},
    EguiPluginHandle,
};

use crate::{connect, fetch_data, State};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Default, Clone)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub full_name: String,

    #[serde(default)]
    pub username: String,

    #[serde(default)]
    pub groups: Vec<String>,

    #[serde(default)]
    pub theme: Theme,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Default, Copy, Clone)]
#[non_exhaustive]
pub(crate) enum Theme {
    #[default]
    Win98,
    ClassicQ3,
    Modern,
}

struct SettingsApp {
    settings: Settings,
    on_submit: Arc<dyn Fn() + Send + Sync>,
}

impl SettingsApp {
    fn new(settings: Settings, on_submit: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            settings,
            on_submit,
        }
    }

    fn on_submit(&self) {
        (self.on_submit)()
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Settings {
                email,
                full_name,
                username,
                groups,
                theme,
            } = &mut self.settings;

            let mut groups_text = groups.join(",");

            ui.label(RichText::new("Settings").font(FontId::proportional(32.0)));

            ui.label("Full Name");
            let _full_name_field = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(full_name),
            );

            ui.label("Email");
            let _email_field = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(email),
            );

            ui.label("Username");
            let _username_field = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(username),
            );

            ui.label("Groups");
            let _groups_field = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(&mut groups_text),
            );

            ui.label("Theme");
            egui::ComboBox::from_id_source("theme_dropdown")
                .width(ui.available_width() * 0.8)
                .selected_text(format!("{:?}", theme))
                .show_ui(ui, |ui| {
                    ui.selectable_value(theme, Theme::Win98, "Windows 98");
                    ui.selectable_value(theme, Theme::ClassicQ3, "ClassicQ3");
                    // ui.selectable_value(theme, Theme::Win98, "Windows 98")
                });

            if ui.button("Submit").clicked() {
                *groups = groups_text.split(',').map(ToOwned::to_owned).collect();

                confy::store("rq", None, self.settings.clone()).unwrap();
                frame.close();
                self.on_submit();
            }
        });
    }
}

impl Settings {
    pub fn launch(self, app: AppHandle) {
        let egui_handle = app.state::<EguiPluginHandle>();

        let native_options = eframe::NativeOptions {
            resizable: false,
            initial_window_size: Some(Vec2::new(500.0, 400.0)),
            ..Default::default()
        };

        let app = app.clone();
        let _window = egui_handle
            .create_window(
                "settings".to_owned(),
                Box::new(move |_| {
                    Box::new(SettingsApp::new(
                        self,
                        Arc::new(move || {
                            let app = app.clone();
                            let state = app.state::<State>();
                            state.cancel_websockets.notify_waiters();
                            tauri::async_runtime::spawn(async move {
                                let state = app.state();
                                let app = app.clone();
                                let _ = connect(app.clone()).await;
                                let _ = fetch_data(app, state).await;
                            });
                        }),
                    ))
                }),
                "Settings".to_owned(),
                native_options,
            )
            .unwrap();
    }
}
