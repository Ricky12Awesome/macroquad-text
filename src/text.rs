use std::{rc::Rc, slice::Iter, str::Chars};

use macroquad::color::Color;

#[derive(Debug, Clone)]
pub enum Component<'a> {
  Str(&'a str),
  Char(char),
  Color(Color),
}

impl<'a> From<&'a str> for Component<'a> {
  fn from(value: &'a str) -> Self {
    Component::Str(value)
  }
}

impl From<char> for Component<'_> {
  fn from(value: char) -> Self {
    Component::Char(value)
  }
}

impl From<Color> for Component<'_> {
  fn from(value: Color) -> Self {
    Component::Color(value)
  }
}

#[derive(Debug, Clone)]
pub struct ColoredStr<'a> {
  components: Rc<[Component<'a>]>,
}

impl<'a> ColoredStr<'a> {
  pub fn new(components: impl IntoIterator<Item = Component<'a>>) -> Self {
    Self {
      components: components.into_iter().collect(),
    }
  }

  pub fn iter(&'a self) -> ColorStrIter<'a, Iter<Component>> {
    ColorStrIter {
      current_color: None,
      current_chars: None,
      components: self.components.iter(),
    }
  }
}

#[derive(Debug)]
pub struct ColorStrIter<'a, I> {
  current_color: Option<Color>,
  current_chars: Option<Chars<'a>>,
  components: I,
}

impl<'a, I> Iterator for ColorStrIter<'a, I>
where
  I: Iterator<Item = &'a Component<'a>>,
{
  type Item = (char, Option<Color>);

  fn next(&mut self) -> Option<Self::Item> {
    match &mut self.current_chars {
      Some(chars) => match chars.next() {
        Some(c) => Some((c, self.current_color)),
        None => {
          self.current_chars = None;
          self.next()
        },
      },
      None => {
        let component = self.components.next()?;
        match component {
          Component::Str(str) => {
            self.current_chars = Some(str.chars());
            self.next()
          }
          &Component::Char(c) => Some((c, self.current_color)),
          &Component::Color(color) => {
            self.current_color = Some(color);
            self.next()
          }
        }
      }
    }
  }
}
