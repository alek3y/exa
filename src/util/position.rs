use std::cmp::{PartialOrd, Ordering};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position {
	pub line: usize,
	pub column: usize
}

impl Position {
	pub fn new(line: usize, column: usize) -> Self {
		Self {line, column}
	}
}

impl PartialOrd for Position {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		if other == self {
			return Some(Ordering::Equal);
		}

		if other.line < self.line {
			return Some(Ordering::Greater);
		}

		if other.line == self.line && other.column < self.column {
			return Some(Ordering::Greater);
		}

		Some(Ordering::Less)
	}
}
