#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Point {
	pub row: u16,
	pub column: u16
}

impl Point {
	pub fn new(row: u16, column: u16) -> Self {
		Self {row, column}
	}

	pub fn origin() -> Self {
		Self {row: 0, column: 0}
	}
}
