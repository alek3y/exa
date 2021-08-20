use super::position::Position;

#[derive(Debug)]
pub struct Region {
	pub top_left: Position,
	pub bottom_right: Position
}

impl Region {
	pub fn new(top_left: Position, bottom_right: Position) -> Self {
		Self {top_left, bottom_right}
	}
}
