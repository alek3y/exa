use std::{fs, io};
use toml::{self, value};

#[derive(Debug)]
pub struct Config(Option<value::Value>);

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

	pub fn unwrap<T>(&self, convert: impl FnOnce(&value::Value) -> Option<T>) -> T {
		convert(self.0.as_ref().unwrap()).expect("wrong `convert` function type")
	}

	pub fn unwrap_or<T>(&self, convert: impl FnOnce(&value::Value) -> Option<T>, default: T) -> T {
		match self.0.as_ref() {
			Some(value) => convert(value),
			None => Some(default)
		}.expect("wrong `convert` function type")
	}
}

impl From<Option<value::Value>> for Config {
	fn from(value: Option<value::Value>) -> Self {
		Self(value)
	}
}
