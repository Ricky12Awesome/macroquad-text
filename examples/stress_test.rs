use macroquad::prelude::*;
use macroquad_text::Fonts;

const NOTO_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");

#[macroquad::main("Stress Test")]
async fn main() {
  let mut font = Fonts::default();

  font
    .load_font_from_bytes_with_scale("Noto Sans", NOTO_SANS, 1000.0)
    .unwrap();

  let mut str = String::new();

  for c in 'a'..='f' {
    str.push(c)
  }

  println!("{str}");

  loop {
    let time = (get_time() / 2.).sin().abs() as f32;

    let text = font.draw_scaled_text(
      &str,
      10.,
      10.,
      20.,
      1. + time,
      Color::from_rgba(220, 220, 220, 255),
    );

    let text = font.draw_scaled_text(
      "E",
      10. + text.width,
      10.,
      20.,
      1. + time,
      Color::from_rgba(200, 50, 20, 255),
    );

    next_frame().await;
  }
}
