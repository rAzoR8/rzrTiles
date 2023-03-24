use egui_extras::{TableBuilder, Column};
use egui::{RichText, Color32, Sense, Label, Button, Vec2};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    tile_data: Vec<u8>,
    width: u32,
    height: u32,
    palette: [Color32; 4]
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            tile_data: vec![0; 8*8],//(0..8*8).collect(),
            width: 8,
            height: 8,
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
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    pub fn get(&self, x: u32, y: u32) -> u8
    {
        let index = (y*self.width+x) as usize;
        self.tile_data[index]
    }
    pub fn set(&mut self, x: u32, y: u32, val: u8)
    {
        let index = (y*self.width+x) as usize;
        self.tile_data[index] = val;
    }

    pub fn export(&self) -> Vec<u8>
    {
        let mut tiles:Vec<u8> = Vec::with_capacity((self.height*self.width*2) as usize);
        for y in 0..self.height {
            for x in (0..self.width).step_by(8) {
                let mut left: u8 = 0;
                let mut right: u8 = 0;

                for i in 0..8 {
                    let cur = self.get(x+i, y);
                    left |= (cur & 0b01) << (7-i);
                    right |= ( ( cur & 0b10 ) >> 1 ) << (7-i);
                }
                tiles.push(left);
                tiles.push(right);
            }
        }
        tiles
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
        //let Self { tile_data, x, y, palette } = self;

        let input = ctx.input(|i| {
            if i.key_pressed(egui::Key::Num1) { return 0; }
            if i.key_pressed(egui::Key::Num2) { return 1; }
            if i.key_pressed(egui::Key::Num3) { return 2; }
            if i.key_pressed(egui::Key::Num4) { return 3; }
            255u8 // invalid
        });

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
            ui.horizontal(|ui|{
                ui.heading("rzrTiles");
                if ui.button("Reset").clicked() {
                    self.tile_data.fill(0);
                }
            });

            ui.horizontal(|ui|{
                for i in 0..self.palette.len() {
                    let mut newcolor = self.palette[i];
                    ui.label(i.to_string());
                    ui.color_edit_button_srgba(&mut newcolor);            
                    self.palette[i] = newcolor;
                }
            });

            let mut width = self.width.clone() / 8;
            ui.add(egui::Slider::new(&mut width, 1..=8).text(format!("Width ({w})", w=self.width)));
            width *= 8;

            let mut height = self.height.clone() / 8;
            ui.add(egui::Slider::new(&mut height, 1..=8).text(format!("Height ({h})", h=self.height)));
            height *= 8;

            if width != self.width || height != self.height
            {
                let mut new_tiles = vec![0; (width*height) as usize];
                let miny = std::cmp::min(self.height, height);
                let minx = std::cmp::min(self.width, width);

                for i in 0..miny {
                    for j in 0..minx {
                        let dst= (i*width + j) as usize;
                        let src= (i*(self.width) + j) as usize;
                        new_tiles[dst] = self.tile_data[src];
                    }
                }

                self.tile_data = new_tiles;
                self.width = width;
                self.height = height;
            }

            for y in 0..self.height {
                for x in (0..self.width).step_by(8) {
                    let mut left: u8 = 0;
                    let mut right: u8 = 0;

                    for i in 0..8 {
                        let cur = self.get(x+i, y);
                        left |= (cur & 0b01) << (7-i);
                        right |= ( ( cur & 0b10 ) >> 1 ) << (7-i);
                    }

                    ui.horizontal(|ui| {
                        ui.label(format!("{:02X} {:02X}", left, right));
                    });                    
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let cell_size: f32 = 20.0;
            TableBuilder::new(ui)
            .columns(Column::auto(), self.width as usize)
            .striped(false)
            //.resizable(true)
            .auto_shrink([false, false])
            .max_scroll_height(1600.0)
            .body(|mut body| {
                body.ui_mut().spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                for r in 0..self.height{ // r = row
                    body.row( cell_size, |mut row| {
                        for c in 0..self.width { // c = column
                            row.col(|ui| {
                                
                                let index = (r*self.width+c) as usize;
                                let i = self.tile_data[index] % (self.palette.len() as u8);
                                let color = self.palette[i as usize];
                                let text = RichText::new( i.to_string() + " " ).background_color(color).size(cell_size).monospace();
                                //let text = i.to_string();
                                //ui.visuals_mut().code_bg_color = color;
                                //ui.visuals_mut().selection.bg_fill = color;
                                //ui.add_sized([20.0, 20.0]                              
                                
                                let sense = Sense::click().union(Sense::hover());
                                let cell = ui.add( Label::new(text).wrap(false).sense(sense) );
                                if cell.clicked() {
                                    self.tile_data[index] = (i+1) % (self.palette.len() as u8);
                                } else if cell.hovered() && input != 255u8{
                                    self.tile_data[index] = input % (self.palette.len() as u8);
                                }
                            });
                        } 
                    });
                }
                //body.ui_mut().shrink_width_to_current();
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
