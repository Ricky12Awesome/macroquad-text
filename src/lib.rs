use fontdue::{Font, FontResult, FontSettings, Metrics};
use macroquad::color::Color;
use macroquad::prelude::draw_texture;
use macroquad::texture::Texture2D;

#[derive(Default)]
pub struct Fonts {
  fonts: Vec<Font>,
}

impl Fonts {
  pub fn fonts(&self) -> &Vec<Font> {
    &self.fonts
  }

  pub fn load_font_from_bytes(&mut self, bytes: &[u8]) -> FontResult<()> {
    let settings = FontSettings {
      collection_index: self.fonts.len() as u32,
      scale: 40.0,
    };

    let font = Font::from_bytes(bytes, settings)?;

    self.fonts.push(font);

    Ok(())
  }

  pub fn rasterize(&self, c: char, px: f32) -> (Metrics, Vec<u8>) {
    let (font_idx, glyph_idx) = self.fonts.iter()
      .map(|font| font.lookup_glyph_index(c))
      .enumerate()
      .find(|(_, glyph_idx)| *glyph_idx != 0)
      .unwrap_or_default();

    self.fonts[font_idx].rasterize_indexed(glyph_idx, px)
  }

  pub fn draw_text(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
    let mut total_width = 0f32;

    for c in text.chars() {
      let (matrix, bitmap) = self.rasterize(c, size);
      let bytes = bitmap
        .iter()
        .flat_map(|coverage| vec![255, 255, 255, *coverage])
        .collect::<Vec<_>>();

      let texture = Texture2D::from_rgba8(matrix.width as u16, matrix.height as u16, &bytes);

      let y = y * 2.0;
      draw_texture(texture, x + total_width, y + (y - matrix.height as f32), color);

      total_width += matrix.advance_width;
    }
  }
}