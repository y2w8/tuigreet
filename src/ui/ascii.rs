use std::error::Error;

use tui::{
  layout::{Alignment, Rect},
  widgets::{Paragraph},
};

use crate::{
  ui::{Frame},
};

pub fn draw(f: &mut Frame, container: Rect) -> Result<(), Box<dyn Error>> {
  let size = f.size();
  let ascii = include_str!("../../contrib/ascii/niri.txt");

  let container_x = (size.width - container.width) / 2;
  let container_y = (size.height - container.height) / 2;

  let ascii_height: u16 = ascii.lines().count() as u16;
  let gap: u16 = 1;

  let ascii_y = container_y.saturating_sub(ascii_height + gap);
  let ascii_rect = Rect::new(container_x, ascii_y, container.width, ascii_height);

  let paragraph = Paragraph::new(ascii).alignment(Alignment::Center);

  f.render_widget(paragraph, ascii_rect);

  Ok(())
}
