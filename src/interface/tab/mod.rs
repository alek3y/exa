use super::pane::Container;

#[derive(Debug)]
pub struct Tab {
	name: String,
	panes: Container
}
