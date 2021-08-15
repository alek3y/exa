use std::{io, fs};
use std::{path, ops::Range};
use unicode_segmentation::UnicodeSegmentation;
use std::{thread, time};

pub mod position;
pub mod cursor;
use self::{position::Position, cursor::Cursor};

#[derive(Debug)]
pub struct Buffer {
	pub buffer: Vec<u8>,
	cursor: Cursor,
	gap: Range<usize>,
	is_crlf: bool,
	path: path::PathBuf,
}

impl Buffer {
	pub fn new(file: &str) -> io::Result<Self> {
		let path = path::Path::new(file);
		let buffer = if path.exists() {
			fs::read(path)?
		} else {
			Vec::new()
		};

		let mut buffer_from_eof = buffer.iter().rev();
		buffer_from_eof.find(|&&byte| byte == b'\n');

		let mut is_crlf = false;
		if let Some(&byte) = buffer_from_eof.next() {
			if byte == b'\r' {
				is_crlf = true;
			}
		}

		Ok(Self {
			buffer,
			cursor: Cursor::new(Position::new(0, 0), 0),
			gap: 0..0,
			is_crlf,
			path: path.to_path_buf()
		})
	}

	pub fn cursor(&self) -> &Cursor {
		&self.cursor
	}

	// TODO: cursor_place

	pub fn gap(&self) -> &Range<usize> {
		&self.gap
	}

	pub fn gap_len(&self) -> usize {
		self.gap.end - self.gap.start
	}

	// TODO: gap_move, gap_resize, gap_insert

	pub fn is_crlf(&self) -> bool {
		self.is_crlf
	}

	pub fn path(&self) -> &path::Path {
		self.path.as_path()
	}
}
