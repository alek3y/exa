use std::io;
pub mod point;
pub mod size;
pub mod util;
pub mod pane;
pub mod tab;
use self::{point::Point, size::Size, tab::Tab};

pub trait Interface {
	fn draw(&self, stdout: &mut io::Stdout, region: (Point, Size), root: Size) -> anyhow::Result<()>;
}

#[derive(Debug)]
struct Window<'a> {
	tabs: Vec<Tab<'a>>
}
