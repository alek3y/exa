use std::io;
use unicode_segmentation::UnicodeSegmentation;
use crossterm::{queue, cursor, style};
use toml::value;

pub mod marker;
pub mod selection;
use marker::Marker;

use crate::buffer::{Buffer, position::Position};
use super::{point::Point, size::Size, util, Interface};

#[derive(Debug)]
pub struct Pane<'a> {
	pub buffer: Buffer<'a>,
	cursor: Marker,

	view_offset: Marker,
	line_count: usize,

	indent_size: usize,
	linenumbers_show: bool,
	options: &'a value::Value
}

impl<'a> Pane<'a> {
	pub fn new(file: &str, options: &'a value::Value) -> anyhow::Result<Self> {
		let buffer = Buffer::new(file, options)?;
		let cursor = Marker::new(0, Position::new(0, 0));
		let line_count = buffer.buffer.iter().filter(|&&byte| byte == b'\n').count() + 1;

		let linenumbers_options = &options["pane"]["linenumbers"];
		let linenumbers_show = linenumbers_options["show"].as_bool().unwrap();

		let buffer_options = &options["buffer"];
		let indent_size = buffer_options["indent_size"].as_integer().map(|size| {
			if size < 0 {0} else {size}
		}).unwrap() as usize;

		Ok(Self {
			buffer,
			cursor,

			view_offset: cursor,
			line_count,

			indent_size,
			linenumbers_show,
			options
		})
	}

	pub fn cursor_locate(&self, position: Position) -> usize {
		if self.cursor.position == position {
			return self.cursor.offset;
		}

		let mut buffer = self.buffer.buffer.iter();

		let mut line_offset = 0;
		for _ in 0..position.line {
			if let Some(relative_offset) = buffer.position(|&byte| byte == b'\n') {
				line_offset += relative_offset + 1;
			} else {
				break;
			}
		}

		let eol = if let Some(offset) = buffer.position(|&byte| byte == b'\n') {
			if offset > 0 && self.buffer.buffer[offset-1] == b'\r' {
				offset-1
			} else {
				offset
			}
		} else {
			self.buffer.buffer.len()
		};

		let line_text = String::from_utf8_lossy(&self.buffer.buffer[line_offset..eol]);

		let mut column = 0;
		let mut column_offset = 0;
		for grapheme in line_text.graphemes(true) {
			if column == position.column {
				break;
			}

			column_offset += grapheme.len();
			if grapheme.as_bytes() != [0] {
				column += 1;
			}
		}

		line_offset + column_offset
	}

	pub fn cursor_place(&mut self, position: Position) {
		self.cursor.offset = self.cursor_locate(position);
		self.cursor.position = position;
	}

	pub fn insert(&mut self, text: &str) {
		let buffer = &mut self.buffer;

		buffer.gap_move(self.cursor.offset);
		self.cursor.offset = buffer.gap.start;

		if text.len() > buffer.gap_len() {
			buffer.gap_resize(text.len());
		}

		for (i, &byte) in text.as_bytes().iter().enumerate() {
			buffer.buffer[buffer.gap.start + i] = byte;
		}

		let offset = text.len();
		buffer.gap.start += offset;
		self.cursor.position.column += offset;
		self.cursor.offset += offset;
	}
}

impl Interface for Pane<'_> {
	fn draw(&self, stdout: &mut io::Stdout, region: (Point, Size), _: Size) -> anyhow::Result<()> {
		use style::*;

		queue!(stdout, cursor::SavePosition)?;

		let pane_options = &self.options["pane"];
		let pane_colors = (
			pane_options["foreground"].as_str().unwrap(),
			pane_options["background"].as_str().unwrap()
		);

		let linenumbers_options = &pane_options["linenumbers"];
		let linenumbers_colors = (
			linenumbers_options["foreground"].as_str().unwrap(),
			linenumbers_options["background"].as_str().unwrap()
		);

		let linenumbers_suffix = linenumbers_options["suffix"].as_str().unwrap();
		let linenumbers_padding = format!("{}", self.line_count).len() + 1;
		let indent = format!("{:1$}", " ", self.indent_size);

		let pane_width = (region.1.width as usize)
			.saturating_sub(linenumbers_padding)
			.saturating_sub(linenumbers_suffix.len());

		let mut text_offset = self.view_offset.offset;
		let mut buffer = self.buffer.buffer[text_offset..].iter();

		for line in 0..region.1.height {
			queue!(stdout, cursor::MoveTo(region.0.column, region.0.row + line))?;

			if self.linenumbers_show {
				queue!(stdout, SetColors(util::colors_guess(
					linenumbers_colors.0, linenumbers_colors.1
				)))?;

				let line_number = line as usize + self.view_offset.position.line;
				let linenumbers_format = if line_number < self.line_count {
					format!("{:>1$}{2}", line_number + 1, linenumbers_padding, linenumbers_suffix)
				} else {
					format!("{:1$}{2}", " ", linenumbers_padding, linenumbers_suffix)
				};
				queue!(stdout, Print(linenumbers_format))?;

				queue!(stdout, ResetColor)?;
			}

			queue!(stdout, SetColors(util::colors_guess(
				pane_colors.0, pane_colors.1
			)))?;

			let eol = buffer.position(|&byte| byte == b'\n')
				.map(|i| i+text_offset)
				.unwrap_or_else(|| self.buffer.buffer.len());

			let mut text = String::from_utf8_lossy(&self.buffer.buffer[text_offset..eol]).to_string();
			text = text.replace("\r", "").replace("\t", &indent);

			text.truncate(pane_width);
			queue!(stdout, Print(&text))?;

			text_offset = eol;
			if text_offset < self.buffer.buffer.len() {
				text_offset += 1;
			}

			if text.len() < pane_width {
				let empty_space = format!("{:1$}", " ", pane_width - text.len());
				queue!(stdout, Print(empty_space))?;
			}

			queue!(stdout, ResetColor)?;
		}

		queue!(stdout, cursor::RestorePosition)?;
		Ok(())
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
	pub fn new(file: &str, options: &'a value::Value) -> anyhow::Result<Self> {
		Ok(Self {
			view: vec![(None, Some(Pane::new(file, options)?))],
			focused: 0,
			layout: Layout::Vertical,
			options
		})
	}

	pub fn split(&mut self, file: &str, direction: Layout) -> anyhow::Result<()> {
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
	fn draw(&self, stdout: &mut io::Stdout, region: (Point, Size), root: Size) -> anyhow::Result<()> {
		use style::*;

		queue!(stdout, cursor::SavePosition)?;

		let layout_options = &self.options["pane"]["layout"];

		queue!(stdout, SetColors(util::colors_guess(
			layout_options["foreground"].as_str().unwrap(),
			layout_options["background"].as_str().unwrap()
		)))?;

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
							queue!(stdout, Print("|"))?;

							if line < children_size.height-1 {
								queue!(stdout, cursor::MoveDown(1))?;
							}
							queue!(stdout, cursor::MoveLeft(1))?;
						}
					},
					Layout::Horizontal => {
						queue!(stdout, cursor::MoveUp(1))?;

						for _ in 0..children_size.width {
							queue!(stdout, Print("-"))?;
						}
					},
				}
			}

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

		queue!(stdout, ResetColor)?;
		queue!(stdout, cursor::RestorePosition)?;
		Ok(())
	}
}
