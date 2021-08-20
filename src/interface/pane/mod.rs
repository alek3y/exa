use std::io;
use crossterm::{queue, cursor, style};
use super::{position::Position, size::Size, Interface};

#[derive(Debug)]
pub struct Pane {
	rows: Vec<String>,
	line_offset: usize,
}

impl Pane {
	pub fn new() -> Self {
		Self {
			rows: Vec::new(),
			line_offset: 0
		}
	}
}

impl Interface for Pane {
	fn draw(&self, stdout: &mut io::Stdout, origin: Position, size: Size) -> io::Result<()> {
		// TODO: Draw pane and buffer
		Ok(())
	}
}

#[derive(Debug)]
pub enum Layout {
	Vertical,
	Horizontal
}

#[derive(Debug)]
pub struct Container {
	view: Vec<(Option<Container>, Option<Pane>)>,
	focused: usize,
	layout: Layout
}

impl Container {
	pub fn new() -> Self {
		Self {
			view: vec![(None, Some(Pane::new()))],
			focused: 0,
			layout: Layout::Vertical
		}
	}

	pub fn split(&mut self, direction: Layout) {
		let focused_view = &mut self.view[self.focused];
		assert!(focused_view.0.is_some() ^ focused_view.1.is_some());

		if focused_view.1.is_some() {
			let mut pane = Pane::new();
			pane.line_offset = 1;
			self.view.push((None, Some(pane)));		// TODO: Check for direction and layout
			return;
		}

		focused_view.0.as_mut().unwrap().split(direction);
	}
}

impl Interface for Container {
	fn draw(&self, stdout: &mut io::Stdout, origin: Position, size: Size) -> io::Result<()> {
		queue!(stdout, cursor::SavePosition)?;

		let children_size = Size::new(size.width/(self.view.len() as u16)-1, size.height);		// TODO: Check for layout

		let mut offset = origin;
		for (i, child) in self.view.iter().enumerate() {
			if i > 0 {
				queue!(stdout, cursor::MoveTo(offset.column-1, offset.row))?;
				for _ in 0..children_size.height {
					queue!(stdout, style::Print("|"))?;
					queue!(stdout, cursor::MoveDown(1), cursor::MoveLeft(1))?;
				}
			}

			if let Some(pane) = &child.1 {
				pane.draw(stdout, offset, children_size);
			} else if let Some(container) = &child.0 {
				container.draw(stdout, offset, children_size);
			}

			// TODO: Check for layout here too
			//offset.row += size.height;
			offset.column += children_size.width+1;
		}

		queue!(stdout, cursor::RestorePosition)
	}
}
