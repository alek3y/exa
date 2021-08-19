use std::io;
pub mod position;
pub mod region;
pub mod pane;
pub mod tab;
use tab::Tab;

pub trait Interface {
	fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()>;
}

#[derive(Debug)]
struct Window {
	tabs: Vec<Tab>
}
