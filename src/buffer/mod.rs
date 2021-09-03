use std::{fs, path, ops::Range};
use toml::value;

pub mod position;

#[derive(Debug)]
pub struct Buffer<'a> {
	pub buffer: Vec<u8>,
	pub gap: Range<usize>,
	path: path::PathBuf,

	is_crlf: bool,
	options: &'a value::Value
}

impl<'a> Buffer<'a> {
	pub fn new(file: &str, options: &'a value::Value) -> anyhow::Result<Self> {
		let path = path::Path::new(file);
		let buffer = if path.exists() {
			fs::read(path)?
		} else {
			Vec::new()
		};

		let eol_options = &options["buffer"]["newline"];

		let mut is_crlf = false;
		if eol_options["detect"].as_bool().unwrap() {
			let mut buffer_from_eof = buffer.iter().rev();
			buffer_from_eof.find(|&&byte| byte == b'\n');

			if let Some(&byte) = buffer_from_eof.next() {
				is_crlf = byte == b'\r';
			}
		} else {
			is_crlf = eol_options["use_crlf"].as_bool().unwrap();
		}

		Ok(Self {
			buffer,
			gap: 0..0,
			is_crlf,
			path: path.to_path_buf(),
			options
		})
	}

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

	pub fn is_crlf(&self) -> bool {
		self.is_crlf
	}

	pub fn path(&self) -> &path::Path {
		self.path.as_path()
	}

	pub fn save(&self) -> anyhow::Result<()> {
		fs::write(&self.path, &self.buffer)?;
		Ok(())
	}
}
