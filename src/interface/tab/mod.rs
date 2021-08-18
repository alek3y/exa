use super::pane::Pane;

#[derive(Debug)]
pub struct Tab {
	name: String,
	panes: Vec<Pane>
}
