use egui_extras::{TableBuilder, Column};
use egui::{RichText, Color32, Sense, Label, Vec2, Stroke};
use std::fs::{File};
use std::io::{BufWriter, Write, BufReader, Read};
use std::{u8};

const TL_MAGIC: &'static [u8] = &[0x72,0x54, 0x69, 0x6c]; // rTil
const TL_VERSION: &'static [u8] = &[1];

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum TileMode
{
    Y8 = 8,
    Y16 = 16
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    pixels: Vec<u8>,
    #[serde(skip)]
    width: u32,
    #[serde(skip)]
    height: u32,
    #[serde(skip)]
    mode: TileMode,
    palette: [Color32; 4],
    picked_path: String,
    scale: f32,
    instant_save: bool
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            pixels: vec![0; 8*8],
            width: 8,
            height: 8,
            mode: TileMode::Y8,
            palette: [Color32::WHITE, Color32::LIGHT_GRAY, Color32::DARK_GRAY, Color32::BLACK],
            picked_path: String::from("tiles.tl"),
            scale: 1.0,
            instant_save: false
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

    pub fn get(&self, x: u32, y: u32) -> u8
    {
        let index = (y*self.width+x) as usize;
        self.pixels[index]
    }
    pub fn set(&mut self, x: u32, y: u32, val: u8)
    {
        let index = (y*self.width+x) as usize;
        self.pixels[index] = val;
    }

    pub fn export(&self) -> Vec<u8>
    {
        pixels_to_gb_tiles(&self.pixels, self.width, self.height, self.mode)
    }

    /// w and h number of tiles
    pub fn import(&mut self, data: &[u8], w: u32, h: u32)
    {
        self.width = w*8;
        self.height = h*8;
        self.pixels = gb_tiles_to_pixels(data, w, h, self.mode);
    }

    pub fn save_to_disk(&self, path: impl Into<String>)
    {
        let file = match File::create(path.into())
        {
            Ok(file) => file,
            Err(err) => 
            {
                // TODO log error
                println!("save_to_disk: {}", err);
                return;
            }
        };

        let mut f = BufWriter::new( file );
        f.write(TL_MAGIC);
        f.write(TL_VERSION);
        f.write(&[self.mode as u8]);
        let dims =[(self.width / 8) as u8, (self.height / 8) as u8];
        f.write(&dims);
        let data = pixels_to_gb_tiles(&self.pixels, self.width, self.height, self.mode);
        f.write(&data);
    }

    pub fn load_from_disk(&mut self, path: impl Into<String>)
    {
        let mut f = BufReader::new( File::open(path.into()).expect("Failed to open file") );

        let mut magic: [u8;4] = [0,0,0,0];
        let mut version: [u8;1] = [0];
        let mut mode: [u8;1] = [0];
        let mut dims: [u8;2] = [0,0];

        f.read_exact(&mut magic);
        f.read_exact(&mut version);
        f.read_exact(&mut mode);
        f.read_exact(&mut dims);

        if magic == TL_MAGIC && version == TL_VERSION
        {
            let w = dims[0];
            let h = dims[1];

            if mode[0] == (TileMode::Y16 as u8) {
                self.mode = TileMode::Y16;
            } else {
                self.mode = TileMode::Y8;
            }

            let bytes_per_tile = self.mode as u8 * 2;
            let mut tiles: Vec<u8> = vec![0; (w*h*bytes_per_tile) as usize ];
            f.read_exact(&mut tiles);

            self.pixels = gb_tiles_to_pixels(&tiles, w as u32, h as u32, self.mode);
            self.width = (w * 8) as u32;
            self.height = (h * 8) as u32;
        }      
    }

}

/// w and h number of tiles
pub fn gb_tiles_to_pixels(data: &[u8], w: u32, h: u32, mode: TileMode) -> Vec<u8>
{
    let ystep = mode as u32;
    assert_eq!(data.len(), (w*h*ystep*2) as usize); // 2 bytes per row, 8/16 rows per tile

    let height = h*ystep;
    let width = w*8;
    let mut pixels: Vec<u8> = vec![0; (width*height) as usize];
    let mut trow = 0;

    for y in 0..h {
        for x in 0..w {
            for j in 0..ystep { // y_tile
                let left = data[trow];trow += 1;
                let right = data[trow];trow += 1;
                for i in 0..8 { // x_tile
                    let color = ((left >> (7-i)) & 0b1) | ((right >> (7-i)) & 0b1) << 1;
                    let pixel = (y*8+j)*(w*8)+(x*8)+i;
                    pixels[pixel as usize] = color;
                }
            }
        }
    }

    assert_eq!(pixels.len(), (w*h*ystep*8) as usize); // 64/128 pixel per tile (8x8/8x16)

    pixels
}

// w and h in number of pixels
pub fn pixels_to_gb_tiles(data: &[u8], w: u32, h: u32, mode: TileMode) -> Vec<u8>
{
    assert_eq!(data.len(), (w*h) as usize);

    let ystep = mode as u32;
    let num_bytes = (w/4)*h;

    let mut tiles: Vec<u8> = Vec::with_capacity(num_bytes as usize);
    for y_tile in 0..h/ystep {
        for x_tile in 0..w/8 {
            for y in 0..ystep {
                let mut left: u8 = 0;
                let mut right: u8 = 0;
    
                for x in 0..8 {
                    let tile = (y_tile*w*ystep)+y*w+x_tile*8+x;
                    let cur = data[ tile as usize];
                    left |= (cur & 0b01) << (7-x);
                    right |= ( ( cur & 0b10 ) >> 1 ) << (7-x);
                }
                tiles.push(left);
                tiles.push(right);
            }
        }
    }

    let len = tiles.len();
    assert_eq!(len, (w*h/4) as usize);

    tiles
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        egui::SidePanel::left("side_panel").min_width(364.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui|{
                ui.heading("rzrTile:");
                ui.label(&self.picked_path);
            });

            ui.horizontal(|ui|{
                if ui.button("Load").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.picked_path = path.display().to_string();
                        self.load_from_disk(&self.picked_path.clone())
                    }
                }

                if ui.button("Save").clicked(){
                    if self.picked_path.is_empty() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            self.picked_path = path.display().to_string();
                        }
                    }

                    if !self.picked_path.is_empty() {
                        self.save_to_disk(&self.picked_path);
                    }
                }             

                if ui.button("Save As").clicked(){
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.picked_path = path.display().to_string();
                        if !self.picked_path.is_empty() {
                            self.save_to_disk(&self.picked_path);
                        }
                    }
                }

                if ui.button("Roundtrip").clicked()
                {
                    self.save_to_disk("roundtrip.tl");
                    self.load_from_disk("roundtrip.tl");
                }

                if ui.button("Reset").clicked() {
                    self.pixels.fill(0);
                }

                ui.checkbox(&mut self.instant_save, "InstantSave");
            });

            ui.horizontal(|ui|{
                ui.label("BG palette:");
                for i in 0..self.palette.len() {
                    let mut newcolor = self.palette[i];
                    ui.label(i.to_string());
                    ui.color_edit_button_srgba(&mut newcolor);            
                    self.palette[i] = newcolor;
                }
            });

            egui::ComboBox::from_label("TileMode")
            .selected_text(format!("{:?}", self.mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.mode, TileMode::Y8, "8x8");
                ui.selectable_value(&mut self.mode, TileMode::Y16, "8x16");
            });

            ui.add(egui::Slider::new(&mut self.scale, 0.0..=4.0).text("Scale"));

            let mut width = std::cmp::max(self.width.clone() / 8,1);
            ui.add(egui::Slider::new(&mut width, 1..=8).text(format!("Width ({w})", w=self.width)));
            width *= 8;

            let ystep = self.mode as u32;

            let mut height = std::cmp::max(self.height.clone() / ystep, 1);
            ui.add(egui::Slider::new(&mut height, 1..=8).text(format!("Height ({h})", h=self.height)));
            height *= ystep;

            // rescale
            if width != self.width || height != self.height
            {
                let mut new_tiles = vec![0; (width*height) as usize];
                let miny = std::cmp::min(self.height, height);
                let minx = std::cmp::min(self.width, width);

                for i in 0..miny {
                    for j in 0..minx {
                        let dst= (i*width + j) as usize;
                        let src= (i*(self.width) + j) as usize;
                        new_tiles[dst] = self.pixels[src];
                    }
                }

                self.pixels = new_tiles;
                self.width = width;
                self.height = height;
            }

            // print hex
            let mut hex_str = String::new();
            for y in 0..self.height {
                for x in (0..self.width).step_by(8) {
                    let mut left: u8 = 0;
                    let mut right: u8 = 0;

                    for i in 0..8 {
                        let cur = self.get(x+i, y);
                        left |= (cur & 0b01) << (7-i);
                        right |= ( ( cur & 0b10 ) >> 1 ) << (7-i);
                    }

                    hex_str.push_str(&format!("{:02X} {:02X}", left, right));               
                    if x+8 < self.width{
                        hex_str.push(' ');
                    }
                }
                hex_str.push('\n');
            }

            let hex_edit = egui::TextEdit::multiline(&mut hex_str).code_editor().desired_width(ui.available_width());
            if ui.add(hex_edit).changed(){
                let mut y = 0;
                for row in hex_str.split('\n'){
                    let mut x_byte = 0;
                    let mut left: u8 = 0;
                    let mut right: u8 = 0;

                    for byte in row.split(' ') {
                        if let Ok(value) = u8::from_str_radix(byte, 16){
                            if x_byte & 1 == 1 {// odd -> right
                                right = value;
                                let x: u32 = x_byte/2;
                                for i in 0..8 { // x_tile
                                    let color = ((left >> (7-i)) & 0b1) | ((right >> (7-i)) & 0b1) << 1;
                                    let pixel = y*self.width + x*8 + i;
                                    self.pixels[pixel as usize] = color;
                                }
                            } else {
                                left = value;
                            }
                        }
                        x_byte += 1;
                    }
                    y += 1;
                }
            }
        });

        let mut changed = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            let cell_size: f32 = 20.0 * self.scale;
            TableBuilder::new(ui)
            .columns(Column::auto_with_initial_suggestion(cell_size), self.width as usize)
            .striped(false)
            .vertical_scroll_offset(1.0)
            //.resizable(true)
            .auto_shrink([false, false])
            .max_scroll_height(1600.0)
            .body(|mut body| {
                body.ui_mut().spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                body.rows(cell_size, self.height as usize, |row_index, mut row|{
                    let r = row_index as u32;
                    for c in 0..self.width {
                        row.col(|ui|{
                            let index = (r*self.width+c) as usize;
                            let i = self.pixels[index] % (self.palette.len() as u8);
                            let bgcolor = self.palette[i as usize];
                            let mut text = RichText::new( i.to_string() + " " ).background_color(Color32::TRANSPARENT).size(cell_size).monospace();  

                            let mut frame = egui::Frame::none();
                            frame = frame.fill(bgcolor);
                            
                            if r % (self.mode as u32) == 0 || c % 8 == 0{
                                text = text.color(Color32::DARK_BLUE);
                            }

                            frame.show(ui, |ui| {
                                let sense = Sense::click().union(Sense::hover());
                                let cell = ui.add( Label::new(text).wrap(false).sense(sense) );
                                let prev = self.pixels[index];
                                if cell.clicked() {
                                    self.pixels[index] = (i+1) % (self.palette.len() as u8);
                                } else if cell.hovered() && input != 255u8{
                                    self.pixels[index] = input % (self.palette.len() as u8);
                                }
                                changed |= self.pixels[index] != prev;
                            });
                        });
                    }
                });
            });
        });

        if changed && self.instant_save && !self.picked_path.is_empty() {
            self.save_to_disk(&self.picked_path);
        }

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
