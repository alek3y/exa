use super::position::Position;

#[derive(Debug)]
pub struct Region {
	pub corners: (Position, Position)
}

impl Region {
	pub fn contains(&self, position: Position) -> bool {
		position >= self.corners.0 && position <= self.corners.1
	}
}
