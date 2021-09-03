use crate::buffer::position::Position;

#[derive(Copy, Clone, Debug)]
pub struct Marker {
	pub offset: usize,
	pub position: Position
}

impl Marker {
	pub fn new(offset: usize, position: Position) -> Self {
		Self {offset, position}
	}
}
