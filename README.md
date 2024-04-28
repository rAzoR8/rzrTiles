# rzrTiles

[GB tile data](https://gbdev.io/pandocs/Tile_Data.html) editor (ab)using [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)

![rzrTiles](assets/rzrtiles.gif)

## File format
```
u32 magic   = "rTiL" (0x72,0x54,0x69,0x6c)
u8  version = 0
u8  mode    = Y8 (height x 8), Y16 (height x 16)
u8  width   = * 8 pixel
u8  height  = * 8 pixel

u8  data[w*h*mode*2]
```