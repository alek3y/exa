#[derive(Debug)]
pub struct Border {
	pub corners: Box<[char; 4]>,
	pub borders: Box<[char; 2]>
}

impl Border {
	pub fn new(corners: [char; 4], borders: [char; 2]) -> Self {
		Self {corners: Box::new(corners), borders: Box::new(borders)}
	}
}
