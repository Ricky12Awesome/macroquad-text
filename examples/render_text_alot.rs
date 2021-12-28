use macroquad::prelude::*;

use macroquad_font_renderer::Fonts;

const NOTO_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
const NOTO_SANS_JP: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.otf");

fn window_conf() -> Conf {
  Conf {
    window_title: "Rendering Text A lot Example".to_owned(),
    window_width: 2000,
    window_height: 1200,
    high_dpi: true,
    window_resizable: true,
    ..Default::default()
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  let mut fonts = Fonts::default();

  fonts.load_font_from_bytes(NOTO_SANS).unwrap();
  fonts.load_font_from_bytes(NOTO_SANS_JP).unwrap();

  for font in fonts.fonts() {
    println!("{} {}", font.file_hash(), font.glyph_count());
  }

  // This might take a while to cache all of these chars

  let chars = (0..24000u32)
    .filter_map(char::from_u32)
    .filter(|c| fonts.contains(*c))
    .collect::<Vec<char>>()
    .chunks(120)
    .map(|it| it.iter().collect::<String>())
    .collect::<Vec<_>>();

  loop {
    for (i, line) in chars.iter().enumerate() {
      fonts.draw_text(line, 0.0, 24.0 * i as f32, 18, Color::from([1.0; 4]));
    }

    next_frame().await;
  }
}




