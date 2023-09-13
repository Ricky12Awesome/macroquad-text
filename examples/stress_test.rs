use macroquad::{color::hsl_to_rgb, prelude::*};
use macroquad::miniquad::conf::Platform;

use macroquad_text::{
  Fonts,
  text::{ColoredStr, Component}, TextParams,
};

const NOTO_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");

fn window_conf() -> Conf {
  Conf {
    window_height: 900,
    window_width: 1600,
    platform: Platform {
      swap_interval: Some(0),
      ..Default::default()
    },
    ..Default::default()
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  let mut font = Fonts::default();

  font
    .load_font_from_bytes_with_scale("Noto Sans", NOTO_SANS, 100.0)
    .unwrap();

  let mut old_time = get_time();
  let mut scale = 0.;

  loop {
    let delta_time = get_time() - old_time;

    scale += 2. * delta_time as f32;

    let mut vec = Vec::new();

    let range = '!'..='~';
    let len = range.size_hint().0 as f32;

    for (index, c) in range.enumerate() {
      let h = (1. / len) * index as f32;
      let h = (scale + h) % 1.;
      let rgb = hsl_to_rgb(h, 1.0, 0.5);
      vec.push(Component::Color(rgb));
      vec.push(Component::Char(c));
    }

    let str = ColoredStr::new( vec);

    font.draw_colored_text_ex(
      &str,
      &TextParams {
        x: 10.,
        y: screen_height() / 2.,
        size: 100.,
        scale: 0.35,
        color: Color::from_rgba(220, 220, 220, 255),
        ..Default::default()
      },
    );

    old_time = get_time();
    next_frame().await;
  }
}
