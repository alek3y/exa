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

	pub fn cursor_locate(text: &str, position: Position, offset: Option<Cursor>) -> Result<Cursor, Cursor> {
		let mut cursor = offset.unwrap_or(Cursor::new(Position::new(0, 0), 0));
		let text_offset = cursor.offset;

		for (i, grapheme) in text.grapheme_indices(true) {
			cursor.offset = text_offset + i;
			if cursor.position == position {
				return Ok(cursor);
			}

			cursor.position.column += 1;
			if grapheme.contains('\n') {

				// Consider EOL a valid location if the column couldn't be reached
				if cursor.position.line == position.line {
					cursor.position.column -= 1;
					return Ok(cursor);
				}

				cursor.position.line += 1;
				cursor.position.column = 0;
			}
		}

		Err(cursor)
	}

	// TODO: Refactor. Do I need `last_found`?
	pub fn cursor_place(&mut self, to_where: Position) {
		if to_where == self.cursor.position {
			return;
		}

		let mut last_found: Cursor;
		{
			let before_gap = String::from_utf8_lossy(&self.buffer[..self.gap.start]);
			match Self::cursor_locate(&before_gap, to_where, None) {
				Ok(cursor) => {
					self.cursor = cursor;
					return;
				},
				Err(cursor) => {
					last_found = cursor;
				}
			}
		}

		last_found.offset = self.gap.end;
		{
			let after_gap = String::from_utf8_lossy(&self.buffer[self.gap.end..]);
			match Self::cursor_locate(&after_gap, to_where, Some(last_found)) {
				Ok(cursor) => {
					self.cursor = cursor;
				},
				Err(mut cursor) => {

					// Set to EOF if it couldn't even match the line
					if cursor.position.line != to_where.line {
						cursor.offset = self.buffer.len();

					// When it errors at EOF move after the last character
					} else if cursor.offset == self.buffer.len() - 1 {
						cursor.offset += 1;
					}

					self.cursor = cursor;
				}
			}
		}
	}

	// TODO: cursor_move (relative)

	unsafe fn chunk_move(&mut self, chunk: Range<usize>, to_where: usize) {
		if chunk.is_empty() || chunk.start == to_where || chunk.end == to_where {
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
		}
	}

	pub fn gap(&self) -> &Range<usize> {
		&self.gap
	}

	pub fn gap_len(&self) -> usize {
		self.gap.end - self.gap.start
	}

	pub fn gap_move(&mut self, to_where: usize) {
		if self.gap.contains(&to_where) || self.gap.end == to_where {
			return;
		}

		let gap_length = self.gap_len();
		if gap_length == 0 {
			self.gap = to_where..to_where;
			return;
		}

		unsafe {
			if to_where < self.gap.start {
				self.chunk_move(to_where..self.gap.start, self.gap.end);
				self.gap = to_where..to_where+gap_length;
			} else {
				self.chunk_move(self.gap.end..to_where, self.gap.start);
				self.gap = to_where-gap_length..to_where;
			}
		}
	}

	pub fn gap_resize(&mut self, to_size: usize) {
		let gap_length = self.gap_len();
		if gap_length == to_size {
			return;
		}

		let buffer_length = self.buffer.len();
		unsafe {
			if to_size > gap_length {
				self.buffer.resize(buffer_length + (to_size - gap_length), 0);
				self.chunk_move(self.gap.end..buffer_length, self.buffer.len());
			} else {
				self.chunk_move(self.gap.end..buffer_length, self.gap.start + to_size);
				self.buffer.truncate(buffer_length - (gap_length - to_size));
			}
		}

		self.gap.end = self.gap.start + to_size;
	}

	// TODO: gap_delete (range?) and gap_insert (assert cursor.position != gap)

	pub fn is_crlf(&self) -> bool {
		self.is_crlf
	}

	pub fn path(&self) -> &path::Path {
		self.path.as_path()
	}
}
