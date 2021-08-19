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
// TODO: impl Interface for Pane
