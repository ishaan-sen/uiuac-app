#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Default)]
struct App {
    state: AppState,
    signon_key: Option<u64>,
    profile: Option<String>,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Uiua386".into(),
            egui::FontData::from_static(include_bytes!("../assets/Uiua386Color.ttf")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "Uiua386".into());
        cc.egui_ctx.set_fonts(fonts);
        Self::default()
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::Explorer {
                target,
                code,
                compiled,
            } => {
                egui::Panel::top("settings_and_profile").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(target, Target::Interpreter, "Interpreted");
                        ui.selectable_value(target, Target::Uasm, "Uasm");
                        ui.selectable_value(target, Target::Uir, "UIR");
                        ui.selectable_value(target, Target::Tir, "TIR");
                        if ui.button("Compile").clicked() {
                            *compiled = Some(String::from("Pretend this is TIR"));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            match &self.profile {
                                Some(name) => ui.button(name),
                                None => ui.button("Sign in"),
                            }
                        });
                    });
                });
                egui::Panel::right("compiled")
                    .default_size(ui.available_width() / 2.0)
                    .show(ui, |ui| {
                        ui.label("Compiled:");
                        if let Some(output) = compiled {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        ui.available_size(),
                                        egui::TextEdit::multiline(output).interactive(false),
                                    )
                                });
                        } else {
                            ui.add_sized(
                                ui.available_size(),
                                egui::Label::new("No compiled code to show"),
                            );
                        }
                    });
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.label("Code Editor:");
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(code).code_editor(),
                            )
                        });
                });
            }
            AppState::Profile { snippets } => todo!(),
        }
    }
}

enum AppState {
    Explorer {
        target: Target,
        code: String,
        compiled: Option<String>,
    },
    Profile {
        snippets: Option<Vec<Snippet>>,
    },
}
impl Default for AppState {
    fn default() -> Self {
        Self::Explorer {
            target: Target::default(),
            code: String::default(),
            compiled: None,
        }
    }
}

#[derive(Default)]
struct Snippet {
    name: String,
    contents: String,
}

#[derive(Default, PartialEq)]
enum Target {
    Interpreter,
    Uasm,
    Uir,
    #[default]
    Tir,
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "Uiuac",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
