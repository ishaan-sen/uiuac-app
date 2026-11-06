#![allow(unused_variables)]
#![allow(dead_code)]

struct App {
    state: AppState,
    signon_key: Option<u64>,
    profile: Option<String>,
    target: Target,
    code: String,
    compiled: Result<String, OutputErr>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            state: Default::default(),
            signon_key: Default::default(),
            profile: Default::default(),
            target: Default::default(),
            code: Default::default(),
            compiled: Err(OutputErr::NoCode),
        }
    }
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
            AppState::Explorer => {
                egui::Panel::top("settings_and_profile").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.target, Target::Interpreter, "Interpreted");
                        ui.selectable_value(&mut self.target, Target::Uasm, "Uasm");
                        ui.selectable_value(&mut self.target, Target::Uir, "UIR");
                        ui.selectable_value(&mut self.target, Target::Tir, "TIR");
                        if ui.button("Compile").clicked() {
                            let agent = ureq::Agent::config_builder()
                                .http_status_as_error(false)
                                .build()
                                .new_agent();
                            let mut response = agent
                                .post("http://localhost:3000/compile")
                                .header("content-type", "application/json")
                                .send(format!(
                                    "{{\"code\": \"{}\", \"emit\": \"{}\"}}",
                                    self.code.replace("\n", "\\n").replace("\"", "\\\""),
                                    self.target.to_string()
                                ))
                                .unwrap();
                            self.compiled = match response.status().as_u16() {
                                x @ 200..=299 => response
                                    .body_mut()
                                    .read_to_string()
                                    .map_err(OutputErr::ReqErr),
                                422 => Err(OutputErr::Compiler(
                                    response.body_mut().read_to_string().unwrap(),
                                )),
                                x => Err(OutputErr::ReqErr(ureq::Error::StatusCode(x))),
                            }
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
                        match &mut self.compiled {
                            Ok(output) => {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.add_sized(
                                            ui.available_size(),
                                            egui::TextEdit::multiline(output)
                                                .code_editor()
                                                .interactive(false),
                                        )
                                    });
                            }
                            Err(OutputErr::Compiler(err)) => {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.add_sized(
                                            ui.available_size(),
                                            egui::TextEdit::multiline(
                                                &mut strip_ansi_escapes::strip_str(err),
                                            )
                                            .code_editor()
                                            .interactive(false),
                                        )
                                    });
                            }
                            Err(err) => {
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::Label::new(format!("{err}")),
                                );
                            }
                        }
                    });
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.label("Code Editor:");
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut self.code).code_editor(),
                            )
                        });
                });
            }
            AppState::Profile { snippets } => todo!(),
        }
    }
}

#[derive(Default)]
enum AppState {
    #[default]
    Explorer,
    Profile {
        snippets: Vec<Snippet>,
    },
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
impl ToString for Target {
    fn to_string(&self) -> String {
        match self {
            Target::Interpreter => todo!(),
            Target::Uasm => "uasm",
            Target::Uir => "uir",
            Target::Tir => "tir",
        }
        .to_owned()
    }
}

enum OutputErr {
    NoCode,
    ReqErr(ureq::Error),
    Compiler(String),
}
impl std::fmt::Display for OutputErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputErr::NoCode => write!(f, "There is no code to show."),
            OutputErr::ReqErr(error) => write!(f, "Request Error: {error}"),
            OutputErr::Compiler(error) => write!(f, "Compiler Error: {error}"),
        }
    }
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
