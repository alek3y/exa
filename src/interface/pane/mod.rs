use crate::util::region::Region;
use super::Interface;

#[derive(Debug)]
pub struct Pane {
	region: Region,
	rows: Vec<String>,
	line_offset: usize
}

// TODO: impl Interface for Pane
