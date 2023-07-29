use rug::{ops::CompleteRound, Assign};

use crate::{app, precision::PRECISION};

pub struct Input {
    pub mouse_drag: Option<glam::f32::Vec2>,
    pub mouse_scroll: Option<f32>,
}

pub struct Interface {
    info_pane: InfoPane,
    position_toolbar: PositionToolbar,
}

pub struct InfoPane;

pub struct PositionToolbar {
    real: LargeFloatEditor,
    imag: LargeFloatEditor,
    zoom: LargeFloatEditor,
}

pub struct LargeFloatEditor {
    value: String,
    editing: bool,
}

impl Interface {
    pub fn new() -> Self {
        Self {
            info_pane: InfoPane,
            position_toolbar: PositionToolbar {
                real: LargeFloatEditor::new(),
                imag: LargeFloatEditor::new(),
                zoom: LargeFloatEditor::new(),
            },
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, globals: &mut app::Globals) -> Input {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let input = ui.input(|i| Input {
                    mouse_drag: {
                        if i.pointer.is_decidedly_dragging() {
                            let (x, y) = i.pointer.delta().into();
                            Some((x, y).into())
                        } else {
                            None
                        }
                    },
                    mouse_scroll: {
                        let scroll = i.scroll_delta;
                        if scroll.y.abs() > 0.0 {
                            Some(scroll.y)
                        } else {
                            None
                        }
                    },
                });

                egui::Window::new("Info")
                    .default_open(true)
                    .show(ctx, |ui: &mut egui::Ui| {
                        self.info_pane.ui(ui, globals);
                    });

                egui::panel::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
                    self.position_toolbar.ui(ui, globals);
                });

                input
            })
            .inner
    }
}

impl InfoPane {
    fn ui(&mut self, ui: &mut egui::Ui, globals: &mut app::Globals) {
        draw_section(ui, "Timing", |ui| {
            ui.label("FPS");
            ui.label(egui::RichText::new(format!("{:.2}", globals.timing.avs_fps)).monospace());

            ui.end_row();

            ui.label("Time");
            ui.label(egui::RichText::new(format!("{:.2}", globals.timing.time)).monospace());
        });
    }
}

impl PositionToolbar {
    fn ui(&mut self, ui: &mut egui::Ui, globals: &mut app::Globals) {
        ui.horizontal(|ui| {
            {
                let real = globals.center.mut_real();
                ui.label(egui::RichText::new("r").monospace());
                self.real.ui(ui, real);
            }

            ui.separator();

            {
                let imag = globals.center.mut_imag();
                ui.label(egui::RichText::new("i").monospace());
                self.imag.ui(ui, imag);
            }

            ui.separator();

            {
                let zoom = &mut globals.zoom;
                ui.label(egui::RichText::new("zoom").monospace());
                self.zoom.ui(ui, zoom);
            }
        });
    }
}

impl LargeFloatEditor {
    fn new() -> Self {
        Self {
            value: String::new(),
            editing: false,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, float: &mut rug::Float) {
        ui.horizontal(|ui| {
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

            if self.editing {
                let lost_focus = ui.text_edit_singleline(&mut self.value).lost_focus();
                if lost_focus && enter_pressed {
                    match rug::Float::parse(&self.value) {
                        Ok(f) => {
                            float.assign(f.complete(PRECISION));
                            self.editing = false;
                        }
                        Err(_) => {}
                    }
                } else if lost_focus {
                    self.editing = false;
                }
            } else {
                self.value = float.to_string();
                self.editing = ui.text_edit_singleline(&mut self.value).changed();
            }
        });
    }
}

fn draw_section<F>(ui: &mut egui::Ui, name: &'static str, builder: F)
where
    F: FnOnce(&mut egui::Ui),
{
    egui::CollapsingHeader::new(name)
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new(name)
                .striped(true)
                .spacing([10.0, 10.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    builder(ui);
                });
        });
}
