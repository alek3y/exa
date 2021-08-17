use super::position::Position;

#[derive(Debug)]
pub struct Cursor {
	pub position: Position,
	pub offset: usize
}

impl Cursor {
	pub fn new(position: Position, offset: usize) -> Self {
		Self {position, offset}
	}
}
