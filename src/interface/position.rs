use crate::buffer;
use std::cmp::{PartialOrd, Ordering};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position {
	pub row: u16,
	pub column: u16
}

impl Position {
	pub fn new(row: u16, column: u16) -> Self {
		Self {row, column}
	}
}

impl PartialOrd for Position {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		buffer::position::Position::new(self.row as usize, self.column as usize).partial_cmp(
			&buffer::position::Position::new(other.row as usize, other.column as usize)
		)
	}
}
