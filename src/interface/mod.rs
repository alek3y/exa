use std::io;
pub mod position;
pub mod size;
pub mod pane;
pub mod tab;
use self::{position::Position, size::Size, tab::Tab};

pub trait Interface {
	fn draw(&self, stdout: &mut io::Stdout, origin: Position, size: Size) -> io::Result<()>;
}

#[derive(Debug)]
struct Window {
	tabs: Vec<Tab>
}
