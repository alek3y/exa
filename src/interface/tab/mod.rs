use super::pane::Container;

#[derive(Debug)]
pub struct Tab<'a> {
	name: String,
	panes: Container<'a>
}
