use egui_extras::{TableBuilder, Column};
use egui::{RichText, Color32, Sense, Label, Button, Vec2};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    tile_data: Vec<u8>,
    x: u32,
    y: u32,
    palette: [Color32; 4]
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            tile_data: vec![0; 8*8],//(0..8*8).collect(),
            x: 8,
            y: 8,
            palette: [Color32::WHITE, Color32::LIGHT_GRAY, Color32::DARK_GRAY, Color32::BLACK]
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { tile_data, x, y, palette } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");
            if ui.button("Reset").clicked() {
                tile_data.fill(0);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            TableBuilder::new(ui)
            .columns(Column::auto(), *x as usize)
            .striped(false)
            //.resizable(false)
            .body(|mut body| {
                body.ui_mut().spacing_mut().item_spacing = Vec2::new(1.0, 1.0);
                for r in 0..*y{
                    body.row( 12.0, |mut row| {
                        for c in 0..*x {
                            row.col(|ui| {
                                let index = (r*(*y)+c) as usize;
                                let i = tile_data[index] % (palette.len() as u8);
                                let color = palette[i as usize];
                                let text = RichText::new( i.to_string() ).background_color(color);
                                //let text = i.to_string();
                                //ui.visuals_mut().code_bg_color = color;
                                //ui.visuals_mut().selection.bg_fill = color;
                                if ui.add(Label::new(text).sense(Sense::click())).clicked() {
                                    tile_data[index] = (i+1) % (palette.len() as u8);
                                }
                                //ui.shrink_width_to_current();
                            });
                        } 
                    });
                }
            });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
