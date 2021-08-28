use std::{fs, io, ops};
use toml::{self, value};

#[derive(Debug)]
pub struct Config(pub Option<value::Value>);

impl Config {
	pub fn new(options: value::Value) -> Self {
		Self(Some(options))
	}

	pub fn read(file: &str) -> io::Result<Self> {
		let contents = fs::read_to_string(file)?;
		Ok(Self(toml::from_str(&contents).ok()))
	}

	pub fn get<I: value::Index>(&self, index: I) -> Config {
		self.0.as_ref().and_then(|value| value.get(index).cloned()).into()
	}

	pub fn lookup<I: value::Index>(&self, path: &[I]) -> Config {
		let mut root = self.0.as_ref();

		for key in path.iter() {
			if root.is_none() {
				break;
			}

			let value = root.unwrap().get(key);
			root = value;
		}

		root.cloned().into()
	}

	pub fn value(&self) -> Option<&value::Value> {
		self.0.as_ref()
	}

	pub fn to_integer(&self, default: i64) -> i64 {
		self.value().and_then(|value| value.as_integer()).unwrap_or(default)
	}

	pub fn to_float(&self, default: f64) -> f64 {
		self.value().and_then(|value| value.as_float()).unwrap_or(default)
	}

	pub fn to_bool(&self, default: bool) -> bool {
		self.value().and_then(|value| value.as_bool()).unwrap_or(default)
	}

	pub fn to_string(&self, default: &str) -> String {
		self.value().and_then(|value| value.as_str()).unwrap_or(default).into()
	}
}

impl From<Option<value::Value>> for Config {
	fn from(value: Option<value::Value>) -> Self {
		Self(value)
	}
}
