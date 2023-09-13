//! A lot of stuff was based on macroquads font system,
//! just removed dpi_scaling and font_scaling and added
//! fallback font support
//!
//! **Example**
//!
//! From [examples/render_text.rs](https://github.com/Ricky12Awesome/macroquad-text/blob/main/examples/render_text.rs)
//!
//! ```rs
//! // Include Fonts
//! const NOTO_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
//! const NOTO_SANS_JP: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.otf");
//!
//! // Window Config for macroquad
//! fn window_conf() -> Conf { ... }
//!
//! #[macroquad::main(window_conf)]
//! async fn main() {
//!   // Start by creating a fonts instance to handle all your fonts
//!   let mut fonts = Fonts::default();
//!
//!   // Load fonts, the order you load fonts is the order it uses for lookups
//!   fonts.load_font_from_bytes("Noto Sans", NOTO_SANS).unwrap();
//!   fonts.load_font_from_bytes("Noto Sans JP", NOTO_SANS_JP).unwrap();
//!
//!   loop {
//!     // Draw text
//!     fonts.draw_text("Nice", 20.0, 0.0, 69, Color::from([1.0; 4]));
//!     fonts.draw_text("良い", 20.0, 89.0, 69, Color::from([1.0; 4]));
//!     fonts.draw_text("Nice 良い", 20.0, 178.0, 69, Color::from([1.0; 4]));
//!
//!     next_frame().await;
//!   }
//! }
//! ```
//!
//! ![img.png](https://raw.githubusercontent.com/Ricky12Awesome/macroquad-text/main/examples/render_text_window.png)
//!

#![deny(unsafe_code)]

use std::{cell::RefCell, collections::HashMap, ops::Deref, path::Path};

use fontdue::{FontResult, FontSettings};
use macroquad::prelude::{
  draw_texture_ex, vec2, Color, DrawTextureParams, FilterMode, Image, TextDimensions,
};

use crate::{
  atlas::Atlas,
  misc::{read_file, IoError, IoErrorKind, IoResult},
  text::ColoredStr,
};

pub(crate) mod atlas;
pub(crate) mod misc;
pub mod text;

pub type ScalingMode = FilterMode;
pub type FontdueFont = fontdue::Font;

/// Where to draw from on the screen
///
/// **Default** [DrawFrom::TopLeft]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DrawFrom {
  /// Starts drawing from the bottom left corner
  BottomLeft,
  /// Starts drawing from the top left corner
  ///
  /// this is the default
  TopLeft,
}

impl Default for DrawFrom {
  fn default() -> Self {
    Self::TopLeft
  }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct CharacterInfo {
  pub id: u64,
  pub offset_x: f32,
  pub offset_y: f32,
  pub advance: f32,
}

/// Text parameters for [Fonts::draw_text_ex]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextParams {
  /// x-coordinate of the text
  pub x: f32,
  /// y-coordinate of the text
  pub y: f32,
  /// The size of the text in pixels
  pub size: f32,
  /// What the text should be scaled by,
  /// this can make text look blurry
  /// since it scales the texture not the
  /// font itself for performance reasons
  pub scale: f32,
  /// The color of the text
  pub color: Color,
  /// Where to draw from
  pub draw: DrawFrom,
}

impl Default for TextParams {
  fn default() -> Self {
    Self {
      x: 0.0,
      y: 0.0,
      size: 22.,
      scale: 1.0,
      color: Color::from_rgba(255, 255, 255, 255),
      draw: DrawFrom::TopLeft,
    }
  }
}

/// Stores font data, also stores caches for much faster rendering times
#[derive(Debug)]
pub struct Font<'a> {
  pub name: &'a str,
  font: FontdueFont,
  atlas: RefCell<Atlas>,
  chars: RefCell<HashMap<(char, u16), CharacterInfo>>,
}

impl<'a> Deref for Font<'a> {
  type Target = FontdueFont;

  fn deref(&self) -> &Self::Target {
    &self.font
  }
}

impl<'a> Font<'a> {
  /// Creates a new font with a given name, [fontdue::Font], and [ScalingMode]
  fn new(name: &'a str, font: FontdueFont, mode: ScalingMode) -> Self {
    Self {
      name,
      font,
      atlas: RefCell::new(Atlas::new(mode)),
      chars: RefCell::default(),
    }
  }

  /// Checks if this font contains a given character
  pub fn contains(&self, c: char) -> bool {
    self.lookup_glyph_index(c) != 0
  }

  fn _cache_glyph(&self, c: char, size: u16) -> CharacterInfo {
    let (matrix, bitmap) = self.rasterize(c, size as f32);
    let (width, height) = (matrix.width as u16, matrix.height as u16);

    let id = self.atlas.borrow_mut().new_unique_id();
    let bytes = bitmap
      .iter()
      .flat_map(|coverage| vec![255, 255, 255, *coverage])
      .collect::<Vec<_>>();

    self.atlas.borrow_mut().cache_sprite(
      id,
      Image {
        width,
        height,
        bytes,
      },
    );

    CharacterInfo {
      id,
      offset_x: matrix.xmin as f32,
      offset_y: matrix.ymin as f32,
      advance: matrix.advance_width,
    }
  }

  /// Caches a glyph for a given character with a given font size
  ///
  /// You don't really need to call this function since caching happens automatically
  pub fn cache_glyph(&self, c: char, size: u16) {
    if !self.chars.borrow().contains_key(&(c, size)) {
      let info = self._cache_glyph(c, size);

      self.chars.borrow_mut().insert((c, size), info);
    }
  }

  /// Recaches all cached glyphs, this is expensive to call
  ///
  /// normally you wouldn't need to call this
  pub fn recache_glyphs(&self) {
    for ((c, size), info) in self.chars.borrow_mut().iter_mut() {
      *info = self._cache_glyph(*c, *size);
    }
  }
}

#[derive(Debug)]
pub struct Fonts<'a> {
  fonts: Vec<Font<'a>>,
  index_by_name: HashMap<&'a str, usize>,
  default_sm: ScalingMode,
}

impl<'a> Default for Fonts<'a> {
  /// Creates a new [Fonts] instance to handle all your font
  ///
  /// Same as calling [Fonts::new(ScalingMode::Linear)]
  fn default() -> Self {
    Self::new(ScalingMode::Linear)
  }
}

impl<'a> Fonts<'a> {
  /// Creates a new [Fonts] instance to handle all your fonts with a given [ScalingMode]
  ///
  /// You can also call [Fonts::default] which defaults to [ScalingMode::Linear]
  ///
  /// **Examples**
  ///
  /// With nearest mode
  /// ```rs
  /// let mut fonts = Fonts::new(ScalingMode::Nearest);
  /// ```
  /// With linear mode
  /// ```rs
  /// let mut fonts = Fonts::new(ScalingMode::Linear);
  /// ```
  pub fn new(default_sm: ScalingMode) -> Self {
    Self {
      fonts: Vec::default(),
      index_by_name: HashMap::default(),
      default_sm,
    }
  }

  /// Returns an immutable reference to the
  /// list of fonts that are currently loaded
  pub fn fonts(&self) -> &Vec<Font> {
    &self.fonts
  }

  /// Caches a glyph for a given character with a given font size
  ///
  /// You don't really need to call this function since caching happens automatically
  pub fn cache_glyph(&self, c: char, size: u16) {
    for font in self.fonts.iter() {
      font.cache_glyph(c, size);
    }
  }

  /// Loads font from bytes with a given name and scale
  ///
  ///
  /// What Scale does
  /// ---------------
  /// (copied from [FontSettings::scale](FontSettings))
  ///
  /// The scale in px the font geometry is optimized for. Fonts rendered at
  /// the scale defined here will be the most optimal in terms of looks and performance. Glyphs
  /// rendered smaller than this scale will look the same but perform slightly worse, while
  /// glyphs rendered larger than this will looks worse but perform slightly better. The units of
  /// the scale are pixels per Em unit.
  pub fn load_font_from_bytes_with_scale(
    &mut self,
    name: &'a str,
    bytes: &[u8],
    scale: f32,
  ) -> FontResult<()> {
    let settings = FontSettings {
      collection_index: 0,
      scale,
    };
    let font = FontdueFont::from_bytes(bytes, settings)?;

    self.index_by_name.insert(name, self.fonts.len());
    self.fonts.push(Font::new(name, font, self.default_sm));

    Ok(())
  }

  /// Loads font from bytes with a given name and a default scale of 100.0
  ///
  /// **See** [Self::load_font_from_bytes_with_scale]
  pub fn load_font_from_bytes(&mut self, name: &'a str, bytes: &[u8]) -> FontResult<()> {
    self.load_font_from_bytes_with_scale(name, bytes, 100.0)
  }

  /// Loads font from a file with a given name and path and a default scale of 100.0
  ///
  /// **See** [Self::load_font_from_bytes_with_scale]
  pub fn load_font_from_file(&mut self, name: &'a str, path: impl AsRef<Path>) -> IoResult<()> {
    self.load_font_from_file_with_scale(name, path, 100.0)
  }

  /// Loads font from a file with a given name, path and scale
  ///
  /// **See** [Self::load_font_from_bytes_with_scale]
  pub fn load_font_from_file_with_scale(
    &mut self,
    name: &'a str,
    path: impl AsRef<Path>,
    scale: f32,
  ) -> IoResult<()> {
    let bytes = read_file(path)?;

    self
      .load_font_from_bytes_with_scale(name, &bytes, scale)
      .map_err(|err| IoError::new(IoErrorKind::InvalidData, err))
  }

  /// Unloads a currently loaded font by its index
  ///
  /// This will also re-index all the currently loaded fonts
  pub fn unload_font_by_index(&mut self, index: usize) {
    if self.fonts.len() <= index {
      return;
    }

    self.fonts.remove(index);
    self.index_by_name.clear();

    for (index, font) in self.fonts.iter().enumerate() {
      self.index_by_name.insert(font.name, index);
    }
  }

  /// Unloads a currently loaded font by it name
  ///
  /// This will also re-index all the currently loaded fonts
  pub fn unload_font_by_name(&mut self, name: &str) {
    self.unload_font_by_index(self.get_index_by_name(name).unwrap_or(self.fonts.len()));
  }

  /// Gets a currently loaded font by its index
  pub fn get_font_by_index(&self, index: usize) -> Option<&Font> {
    self.fonts.get(index)
  }

  /// Gets the first currently loaded font if it contains this character
  pub fn get_index_by_char(&self, c: char) -> Option<usize> {
    self.fonts.iter().position(|it| it.contains(c))
  }

  /// Gets a currently loaded font index by its name
  pub fn get_index_by_name(&self, name: &str) -> Option<usize> {
    self.index_by_name.get(name).copied()
  }

  /// Gets a currently loaded font by its name
  pub fn get_font_by_name(&self, name: &str) -> Option<&Font> {
    self.get_font_by_index(self.get_index_by_name(name)?)
  }

  /// Gets the first currently loaded font if it contains this character
  pub fn get_font_by_char(&self, c: char) -> Option<&Font> {
    self.get_font_by_index(self.get_index_by_char(c)?)
  }

  /// Gets the first currently loaded font if it contains this character,
  /// if no font that contains this character is found, it will return the first loaded font,
  /// **if no fonts are loaded then it will panic**
  pub fn get_font_by_char_or_panic(&self, c: char) -> &Font {
    self
      .get_font_by_char(c)
      .or_else(|| self.fonts.first())
      .expect("There is no font currently loaded")
  }

  /// Checks if any fonts supports this character
  pub fn contains(&self, c: char) -> bool {
    self.fonts.iter().any(|f| f.contains(c))
  }

  /// Measures text with a given font size
  ///
  /// **Example**
  /// ```rs
  /// let dimensions = fonts.measure_text("Some Text", 22);
  ///
  /// println!("width: {}, height: {}, offset_y: {}",
  ///   dimensions.width,
  ///   dimensions.height,
  ///   dimensions.offset_y
  /// )
  /// ```
  ///
  /// **See** [TextDimensions]
  pub fn measure_text(&self, text: impl Iterator<Item = char>, size: f32) -> TextDimensions {
    let mut width = 0f32;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for c in text {
      let font = self.get_font_by_char_or_panic(c);

      font.cache_glyph(c, size as u16);

      let info = font.chars.borrow()[&(c, size as u16)];
      let glyph = font.atlas.borrow().get(info.id).unwrap().rect;

      width += info.advance;

      if min_y > info.offset_y {
        min_y = info.offset_y;
      }

      if max_y < glyph.h + info.offset_y {
        max_y = glyph.h + info.offset_y;
      }
    }

    TextDimensions {
      width,
      height: max_y - min_y,
      offset_y: max_y,
    }
  }

  /// Measures text with a given font size and scale
  ///
  /// **Example**
  /// ```rs
  /// let dimensions = fonts.measure_scaled_text("Some Text", 22, 1.5);
  ///
  /// println!("width: {}, height: {}, offset_y: {}",
  ///   dimensions.width,
  ///   dimensions.height,
  ///   dimensions.offset_y
  /// )
  /// ```
  ///
  /// **See** [TextDimensions]
  pub fn measure_scaled_text(
    &self,
    text: impl Iterator<Item = char>,
    size: f32,
    scale: f32,
  ) -> TextDimensions {
    let mut width = 0f32;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for c in text {
      let font = self.get_font_by_char_or_panic(c);

      font.cache_glyph(c, size as u16);

      let info = font.chars.borrow()[&(c, size as u16)];
      let glyph = font.atlas.borrow().get(info.id).unwrap().rect;
      let h = glyph.h * scale;
      let offset_y = info.offset_y * scale;

      width += info.advance * scale;

      if min_y > offset_y {
        min_y = offset_y;
      }

      if max_y < h + offset_y {
        max_y = h + offset_y;
      }
    }

    TextDimensions {
      width,
      height: max_y - min_y,
      offset_y: max_y,
    }
  }

  /// Draws text with a given font size, draws from TopLeft
  ///
  /// **Examples**
  /// ```rs
  /// fonts.draw_text("Some Text", 20.0, 20.0, 22, Color::from_rgba(255, 255, 255, 255));
  /// ```
  ///
  /// **See** [Self::draw_text_ex]
  pub fn draw_text(&self, text: &str, x: f32, y: f32, size: f32, color: Color) -> TextDimensions {
    self.draw_text_ex(
      text,
      &TextParams {
        x,
        y,
        size,
        scale: 1.0,
        color,
        draw: Default::default(),
      },
    )
  }

  /// Draws text with given [TextParams]
  ///
  /// **Example**
  /// ```rs
  /// fonts.draw_text_ex("Some Text", &TextParams {
  ///   x: 20.,
  ///   y: 20.,
  ///   // Default Size
  ///   size: 22.,
  ///   // Default Scale
  //    scale: 1.
  ///   // Default Color
  ///   color: Color::from_rgba(255, 255, 255, 255),
  ///   // Default Draw method
  ///   draw: DrawFrom::TopLeft
  /// });
  ///
  /// // Does the same as above
  /// fonts.draw_text_ex("Some Text", &TextParams {
  ///   x: 20.,
  ///   y: 20.,
  ///   ..Default::default()
  /// });
  /// ```
  ///
  /// **See** [Self::draw_text]
  pub fn draw_text_ex(&self, text: &str, params: &TextParams) -> TextDimensions {
    let mut total_width = 0f32;

    for c in text.chars() {
      let font = self.get_font_by_char_or_panic(c);
      font.cache_glyph(c, params.size as u16);
    }

    for c in text.chars() {
      let font = self.get_font_by_char_or_panic(c);
      let advance = self._draw_char(c, total_width, params.color, font, params);

      total_width += advance;
    }

    self.measure_scaled_text(text.chars(), params.size, params.scale)
  }

  pub fn draw_colored_text_ex(&self, text: &ColoredStr, params: &TextParams) -> TextDimensions {
    let mut total_width = 0f32;

    for (c, _) in text.iter() {
      let font = self.get_font_by_char_or_panic(c);
      font.cache_glyph(c, params.size as u16);
    }

    for (c, color) in text.iter() {
      let color = color.unwrap_or(params.color);
      let font = self.get_font_by_char_or_panic(c);
      let advance = self._draw_char(c, total_width, color, font, params);

      total_width += advance;
    }

    self.measure_scaled_text(text.iter().map(|(c, _)| c), params.size, params.scale)
  }

  fn _draw_char(
    &self,
    c: char,
    current_width: f32,
    color: Color,
    font: &Font,
    params: &TextParams,
  ) -> f32 {
    let mut atlas = font.atlas.borrow_mut();
    let info = &font.chars.borrow()[&(c, params.size as u16)];
    let glyph = atlas.get(info.id).unwrap().rect;
    let w = glyph.w * params.scale;
    let h = glyph.h * params.scale;
    let offset_x = info.offset_x * params.scale;
    let offset_y = info.offset_y * params.scale;
    let advance = info.advance * params.scale;

    let mut y = 0.0 - h - offset_y + params.y;

    if let DrawFrom::TopLeft = params.draw {
      y += params.size * params.scale;
    }

    draw_texture_ex(
      atlas.texture(),
      offset_x + current_width + params.x,
      y,
      color,
      DrawTextureParams {
        dest_size: Some(vec2(w, h)),
        source: Some(glyph),
        ..Default::default()
      },
    );

    advance
  }

  pub fn draw_char(&self, c: char, current_width: f32, params: &TextParams) -> f32 {
    let font = self.get_font_by_char_or_panic(c);
    font.cache_glyph(c, params.size as u16);

    self._draw_char(c, current_width, params.color, font, params)
  }
}
