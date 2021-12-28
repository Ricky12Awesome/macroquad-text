//! Heavily inspired from macroquad font system
//! just with a lot of fluff removed and fallback font support

use std::cell::RefCell;
use std::collections::HashMap;

use fontdue::{Font, FontResult, FontSettings, Metrics};
use macroquad::color::Color;
use macroquad::math::vec2;
use macroquad::miniquad::FilterMode;
use macroquad::prelude::{draw_texture_ex, Image};
use macroquad::texture::DrawTextureParams;

use crate::atlas::Atlas;

pub mod atlas;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DrawFrom {
  BottomLeft,
  TopLeft,
}

impl Default for DrawFrom {
  fn default() -> Self {
    Self::TopLeft
  }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct CharacterInfo {
  pub id: u64,
  pub offset_x: f32,
  pub offset_y: f32,
  pub advance: f32,
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct TextParams {
  pub x: f32,
  pub y: f32,
  pub size: u16,
  pub color: Color,
  pub draw: DrawFrom,
}

#[derive(Default)]
pub struct Fonts {
  fonts: Vec<Font>,
  atlas: RefCell<Atlas>,
  chars: RefCell<HashMap<(char, u16), CharacterInfo>>,
}

impl Fonts {
  pub fn new(filter: FilterMode) -> Self {
    Self {
      atlas: RefCell::new(Atlas::new(filter)),
      ..Self::default()
    }
  }

  pub fn fonts(&self) -> &Vec<Font> {
    &self.fonts
  }

  pub fn cache_glyph(&self, c: char, size: u16) {
    if self.chars.borrow().contains_key(&(c, size)) {
      return;
    }

    let mut cache = self.chars.borrow_mut();
    let mut atlas = self.atlas.borrow_mut();

    let (matrix, bitmap) = self.rasterize(c, size as f32);
    let (width, height) = (matrix.width as u16, matrix.height as u16);

    let id = atlas.new_unique_id();
    let bytes = bitmap
      .iter()
      .flat_map(|coverage| vec![255, 255, 255, *coverage])
      .collect::<Vec<_>>();

    atlas.cache_sprite(id, Image { width, height, bytes });

    let info = CharacterInfo {
      id,
      offset_x: matrix.xmin as f32,
      offset_y: matrix.ymin as f32,
      advance: matrix.advance_width,
    };

    cache.insert((c, size), info);
  }

  pub fn load_font_from_bytes_with_scale(&mut self, bytes: &[u8], scale: f32) -> FontResult<()> {
    let settings = FontSettings { collection_index: 0, scale };
    let font = Font::from_bytes(bytes, settings)?;

    self.fonts.push(font);

    Ok(())
  }

  /// Loads font from bytes with a scale of 100.0
  pub fn load_font_from_bytes(&mut self, bytes: &[u8]) -> FontResult<()> {
    self.load_font_from_bytes_with_scale(bytes, 10000.0)
  }

  /// Checks if any fonts supports this character
  pub fn contains(&self, c: char) -> bool {
    self.lookup_glyph_index(c).1 != 0
  }

  // Looks up glyph index in all fonts until it finds one
  pub fn lookup_glyph_index(&self, c: char) -> (usize, u16) {
    self.fonts.iter()
      .map(|font| font.lookup_glyph_index(c))
      .enumerate()
      .find(|(_, glyph_idx)| *glyph_idx != 0)
      .unwrap_or_default()
  }

  /// Will rasterize a character using which ever font that contains that character first
  ///
  /// **See** [Font::rasterize] or [Font::rasterize_indexed]
  pub fn rasterize(&self, c: char, px: f32) -> (Metrics, Vec<u8>) {
    let (font_idx, glyph_idx) = self.lookup_glyph_index(c);

    self.fonts.get(font_idx)
      .map(|it| it.rasterize_indexed(glyph_idx, px))
      .unwrap_or_default()
  }

  pub fn draw_text(&self, text: &str, x: f32, y: f32, size: u16, color: Color) {
    self.draw_text_ex(text, &TextParams {
      x,
      y,
      size,
      color,
      draw: Default::default(),
    })
  }

  pub fn draw_text_ex(&self, text: &str, params: &TextParams) {
    let mut total_width = 0f32;

    for c in text.chars() {
      self.cache_glyph(c, params.size);
      let mut atlas = self.atlas.borrow_mut();
      let info = &self.chars.borrow()[&(c, params.size)];
      let glyph = atlas.get(info.id).unwrap().rect;
      let mut y = 0.0 - glyph.h - info.offset_y + params.y;

      if let DrawFrom::TopLeft = params.draw {
        y += params.size as f32;
      }

      draw_texture_ex(
        atlas.texture(),
        info.offset_x + total_width + params.x,
        y,
        params.color,
        DrawTextureParams {
          dest_size: Some(vec2(glyph.w, glyph.h)),
          source: Some(glyph),
          ..Default::default()
        },
      );

      total_width += info.advance;
    }
  }
}