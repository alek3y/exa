use crate::buffer;

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
