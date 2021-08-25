use std::{io, fs};
use std::{path, ops::Range};
use unicode_segmentation::UnicodeSegmentation;

pub mod cursor;
pub mod position;
use self::{cursor::Cursor, position::Position};
use crate::config::Config;

#[derive(Debug)]
pub struct Buffer<'a> {
	pub buffer: Vec<u8>,
	cursor: Cursor,
	gap: Range<usize>,
	is_crlf: bool,
	path: path::PathBuf,
	options: &'a Config
}

impl<'a> Buffer<'a> {
	pub fn new(file: &str, options: &'a Config) -> io::Result<Self> {
		let path = path::Path::new(file);
		let buffer = if path.exists() {
			fs::read(path)?
		} else {
			Vec::new()
		};

		let eol_options = options.lookup(&["buffer", "newline"]);

		let mut is_crlf = false;
		if eol_options.get("detect").unwrap_or(|value| value.as_bool(), true) {
			let mut buffer_from_eof = buffer.iter().rev();
			buffer_from_eof.find(|&&byte| byte == b'\n');

			if let Some(&byte) = buffer_from_eof.next() {
				is_crlf = byte == b'\r';
			}
		} else {
			is_crlf = eol_options.get("use_crlf").unwrap_or(|value| value.as_bool(), false);
		}

		Ok(Self {
			buffer,
			cursor: Cursor::new(Position::new(0, 0), 0),
			gap: 0..0,
			is_crlf,
			path: path.to_path_buf(),
			options
		})
	}

	pub fn cursor(&self) -> &Cursor {
		&self.cursor
	}

	pub fn cursor_locate(text: &str, position: Position, offset: Option<Cursor>) -> Result<Cursor, Cursor> {
		let mut cursor = offset.unwrap_or_else(|| Cursor::new(Position::new(0, 0), 0));
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

	pub fn insert(&mut self, text: &str) {
		self.gap_move(self.cursor.offset);
		self.cursor.offset = self.gap.start;		// Otherwise it's at the end of the gap

		if text.len() > self.gap_len() {
			self.gap_resize(text.len());		// TODO: Increase more?
		}

		for (i, &byte) in text.as_bytes().iter().enumerate() {
			self.buffer[self.gap.start + i] = byte;
		}

		let offset = text.len();
		self.gap.start += offset;
		self.cursor.position.column += offset;
		self.cursor.offset += offset;
	}

	// TODO: gap_delete (range?)

	pub fn is_crlf(&self) -> bool {
		self.is_crlf
	}

	pub fn path(&self) -> &path::Path {
		self.path.as_path()
	}

	pub fn save(&self) -> io::Result<()> {
		// TODO: Add \n at EOF automatically?
		fs::write(&self.path, &self.buffer)
	}
}
