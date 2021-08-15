use super::position::Position;

#[derive(Debug)]
pub struct Cursor {
	position: Position,
	offset: usize
}

impl Cursor {
	pub fn new(position: Position, offset: usize) -> Self {
		Self {position, offset}
	}

	pub fn position(&self) -> Position {
		self.position
	}

	pub fn offset(&self) -> usize {
		self.offset
	}
}
