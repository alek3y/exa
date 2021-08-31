use crossterm::terminal;

#[derive(Copy, Clone, Debug)]
pub struct Size {
	pub width: u16,
	pub height: u16
}

impl Size {
	pub fn new(width: u16, height: u16) -> Self {
		Self {width, height}
	}

	pub fn terminal() -> anyhow::Result<Self> {
		let size = terminal::size()?;
		Ok(Self {width: size.0, height: size.1})
	}
}

impl From<(u16, u16)> for Size {
	fn from(size: (u16, u16)) -> Self {
		Self::new(size.0, size.1)
	}
}
