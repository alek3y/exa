pub mod pane;
pub mod tab;
use tab::Tab;

trait Interface {
	fn draw(&self);
}

#[derive(Debug)]
struct Window {
	tabs: Vec<Tab>
}
