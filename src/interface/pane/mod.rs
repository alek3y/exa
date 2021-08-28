use std::io;
use crossterm::{queue, cursor, style::{self, Color}};
use toml::value;

use super::{position::Position, size::Size, Interface};
use crate::buffer::Buffer;

#[derive(Debug)]
pub struct Pane<'a> {
	buffer: Buffer<'a>,
	rows: Vec<String>,
	line_offset: usize,
	options: &'a value::Value
}

impl<'a> Pane<'a> {
	pub fn new(file: &str, options: &'a value::Value) -> io::Result<Self> {
		Ok(Self {
			buffer: Buffer::new(file, options)?,
			rows: Vec::new(),
			line_offset: 0,
			options
		})
	}
}

impl Interface for Pane<'_> {
	fn draw(&self, stdout: &mut io::Stdout, region: (Position, Size), _: Size) -> io::Result<()> {
		queue!(stdout, cursor::SavePosition)?;

		let line_options = &self.options["pane"]["linenumbers"];

		if line_options["enable"].as_bool().unwrap() {
			let line_count = self.buffer.buffer.iter().filter(|&&c| c == b'\n').count() + 1;
			let padding = format!("{}", line_count).len() + 1;
			let suffix = line_options["suffix"].as_str().unwrap();

			queue!(stdout,
				style::SetForegroundColor(
					Color::parse_ansi(line_options["foreground"].as_str().unwrap())
						.unwrap_or(Color::Reset)
				),
				style::SetBackgroundColor(
					Color::parse_ansi(line_options["background"].as_str().unwrap())
						.unwrap_or(Color::Reset)
				)
			)?;

			queue!(stdout, cursor::MoveTo(region.0.column, region.0.row))?;
			for line in 0..region.1.height {
				let line_number = line as usize + self.line_offset;

				let line_format = if line_number < line_count {
					format!("{:>1$}{2}", line_number + 1, padding, suffix)
				} else {
					format!("{:1$}{2}", " ", padding, suffix)
				};
				queue!(stdout, style::Print(line_format))?;

				if line < region.1.height-1 {
					queue!(stdout, cursor::MoveDown(1))?;
				}
				queue!(stdout, cursor::MoveLeft((padding + suffix.len()) as u16))?;
			}

			queue!(stdout, style::ResetColor)?;
		}

		queue!(stdout, cursor::RestorePosition)
	}
}

#[derive(PartialEq, Debug)]
pub enum Layout {
	Vertical,
	Horizontal
}

#[derive(Debug)]
pub struct Container<'a> {
	view: Vec<(Option<Container<'a>>, Option<Pane<'a>>)>,
	pub focused: usize,
	layout: Layout,
	options: &'a value::Value
}

impl<'a> Container<'a> {
	pub fn new(file: &str, options: &'a value::Value) -> io::Result<Self> {
		Ok(Self {
			view: vec![(None, Some(Pane::new(file, options)?))],
			focused: 0,
			layout: Layout::Vertical,
			options
		})
	}

	pub fn split(&mut self, file: &str, direction: Layout) -> io::Result<()> {
		let focused_view = &mut self.view[self.focused];
		assert!(focused_view.0.is_some() ^ focused_view.1.is_some());

		if focused_view.1.is_some() {
			let new_view;
			let mut view_position = self.focused;
			if self.layout == direction {
				new_view = (None, Some(Pane::new(file, self.options)?));
				view_position += 1;
			} else {
				let focused_pane = self.view.remove(self.focused).1.unwrap();

				let mut container = Container::new(file, self.options)?;
				container.layout = direction;
				container.view.insert(0, (None, Some(focused_pane)));
				new_view = (Some(container), None);
			}

			if self.view.is_empty() {
				self.view.push(new_view);
			} else {
				self.view.insert(view_position, new_view);
			}

			return Ok(());
		}

		focused_view.0.as_mut().unwrap().split(file, direction)
	}
}

impl Interface for Container<'_> {
	fn draw(&self, stdout: &mut io::Stdout, region: (Position, Size), root: Size) -> io::Result<()> {
		queue!(stdout, cursor::SavePosition)?;

		let children_amount = self.view.len() as u16;
		let mut children_size = match self.layout {
			Layout::Vertical => Size::new(
				(region.1.width / children_amount).saturating_sub(1),
				region.1.height
			),
			Layout::Horizontal => Size::new(
				region.1.width,
				(region.1.height / children_amount).saturating_sub(1)
			)
		};

		let mut child_offset = region.0;
		for (i, child) in self.view.iter().enumerate() {
			if i > 0 {
				queue!(stdout, cursor::MoveTo(child_offset.column, child_offset.row))?;

				match self.layout {
					Layout::Vertical => {
						queue!(stdout, cursor::MoveLeft(1))?;

						for line in 0..children_size.height {
							queue!(stdout, style::Print("|"))?;

							if line < children_size.height-1 {
								queue!(stdout, cursor::MoveDown(1))?;
							}
							queue!(stdout, cursor::MoveLeft(1))?;
						}
					},
					Layout::Horizontal => {
						queue!(stdout, cursor::MoveUp(1))?;

						for _ in 0..children_size.width {
							queue!(stdout, style::Print("-"))?;
						}
					},
				}
			}

			// TODO: Why does the last one disappear when it goes towards root.height/width?
			if i == self.view.len()-1 {
				match self.layout {
					Layout::Vertical => {
						children_size.width = root.width.saturating_sub(child_offset.column);
					},
					Layout::Horizontal => {
						children_size.height = root.height.saturating_sub(child_offset.row);
					},
				}
			}

			if let Some(pane) = &child.1 {
				pane.draw(stdout, (child_offset, children_size), root)?;
			} else if let Some(container) = &child.0 {
				container.draw(stdout, (child_offset, children_size), root)?;
			}

			match self.layout {
				Layout::Vertical => {
					child_offset.column += children_size.width;

					let max_offset = region.0.column + region.1.width;
					if child_offset.column < max_offset {
						child_offset.column += 1;
					}
				},
				Layout::Horizontal => {
					child_offset.row += children_size.height;

					let max_offset = region.0.row + region.1.height;
					if child_offset.row < max_offset {
						child_offset.row += 1;
					}
				}
			}
		}

		queue!(stdout, cursor::RestorePosition)
	}
}
