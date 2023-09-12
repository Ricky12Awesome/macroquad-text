use macroquad::prelude::*;

use macroquad_text::Fonts;
use macroquad_text::TextParams;

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

    let text = font.draw_text_ex(&str, &TextParams {
      x: 10.,
      y: 10.,
      size: 100.,
      scale: time,
      color: Color::from_rgba(220, 220, 220, 255),
      ..Default::default()
    });

    let _ = font.draw_text_ex("E", &TextParams {
      x: 10. + text.width,
      y: 10.,
      size: 100.,
      scale: time,
      color: Color::from_rgba(180, 20, 30, 255),
      ..Default::default()
    });

    next_frame().await;
  }
}
