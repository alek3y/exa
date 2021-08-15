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

	unsafe fn chunk_move(&mut self, chunk: &Range<usize>, to_where: usize) {
		if to_where == chunk.start {
			return;
		}

		let is_left = to_where < chunk.start;
		let mut bytes = chunk.to_owned();

		while let Some(i) = if is_left {bytes.next()} else {bytes.next_back()} {
			let offsetted_index = if is_left {
				i - chunk.start + to_where
			} else {
				i + to_where - chunk.end
			};

			self.buffer[offsetted_index] = self.buffer[i];
			self.buffer[i] = 0;

			/*
			// TODO: Switch to unsafe code when I'm sure it won't panic lol

			*self.buffer.get_unchecked_mut(offsetted_index) = *self.buffer.get_unchecked(i);
			*self.buffer.get_unchecked_mut(i) = 0;
			*/
		}
	}

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
