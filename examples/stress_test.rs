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

  for c in 'A'..='z' {
    str.push(c)
  }

  println!("{str}");

  loop {
    let time = (get_time() * 25.).sin().abs() as f32;
    let size = 1000. * time;

    font.draw_scaled_text(
      &str,
      10.,
      10.,
      size,
      1000,
      Color::from_rgba(220, 220, 220, 255),
    );

    next_frame().await;
  }
}
