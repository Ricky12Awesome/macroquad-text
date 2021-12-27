use macroquad::prelude::*;

use macroquad_font_renderer::Fonts;

const NOTO_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
const NOTO_SANS_JP: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.otf");

fn window_conf() -> Conf {
  Conf {
    window_title: "Rendering Text Example".to_owned(),
    window_width: 1280,
    window_height: 720,
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

  loop {
    fonts.draw_text("Nice", 20.0, 20.0, 69.0, Color::from([1.0; 4])); // Nice
    fonts.draw_text("良い", 20.0, 50.0, 69.0, Color::from([1.0; 4])); // Nice
    fonts.draw_text("Nice 良い", 20.0, 80.0, 69.0, Color::from([1.0; 4])); // Nice

    next_frame().await;
  }
}





