use std::io;
use crossterm::{queue, cursor, style};
use super::{region::Region, Interface};

pub mod border;
use border::Border;

#[derive(Debug)]
pub struct Pane {
	border: Border,
	region: Region,
	rows: Vec<String>,
	line_offset: usize
}

impl Pane {
	pub fn new(border: Border, region: Region, line_offset: usize) -> Self {
		Self {border, region, rows: Vec::new(), line_offset}
	}
}

impl Interface for Pane {
	fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()> {
		let top_left = self.region.top_left;
		let bottom_right = self.region.bottom_right;

		let borders = &self.border.borders;
		let corners = &self.border.corners;

		queue!(stdout, cursor::SavePosition)?;

		// Top left corner
		queue!(stdout, cursor::MoveTo(top_left.column, top_left.row))?;
		queue!(stdout, style::Print(corners[0]))?;

		// Top border
		for _ in top_left.column..bottom_right.column-1 {
			queue!(stdout, style::Print(borders[0]))?;
		}

		// Top right corner
		queue!(stdout, style::Print(corners[1]))?;

		// Right border
		for _ in top_left.row+1..bottom_right.row {
			queue!(stdout, cursor::MoveLeft(1), cursor::MoveDown(1))?;
			queue!(stdout, style::Print(borders[1]))?;
		}
		
		// Left border
		queue!(stdout, cursor::MoveTo(top_left.column, top_left.row+1))?;
		for _ in top_left.row+1..bottom_right.row {
			queue!(stdout, style::Print(borders[1]))?;
			queue!(stdout, cursor::MoveLeft(1), cursor::MoveDown(1))?;
		}

		// Bottom left corner
		queue!(stdout, style::Print(corners[3]))?;

		// Bottom border
		for _ in top_left.column..bottom_right.column-1 {
			queue!(stdout, style::Print(borders[0]))?;
		}

		// Bottom right corner
		queue!(stdout, style::Print(corners[2]))?;

		queue!(stdout, cursor::RestorePosition)
	}
}
