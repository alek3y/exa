use toml::value::Value;

pub fn default() -> Value {
	toml::toml! {
		[buffer]
		tab_size = 3

		[buffer.newline]
		detect = true
		use_crlf = false

		[pane]
		background = "#1d1f21"
		foreground = "#c5c8c6"

		[pane.layout]
		foreground = "#b1b5b3"
		background = "#282a2e"

		[pane.linenumbers]
		enable = true
		suffix = "|"
		foreground = "#8a8c8b"
		background = "#282a2e"
	}
}

pub fn update(default: &mut Value, changes: &Value) {
	if !default.is_table() || !changes.is_table() {
		return;
	}

	let default = default.as_table_mut().unwrap();
	let changes = changes.as_table().unwrap();

	for (key, changed) in changes.iter() {
		if default.contains_key(key) {
			match default[key] {
				Value::Table(_) => update(default.get_mut(key).unwrap(), changed),
				_ => {
					if default[key].same_type(changed) {
						default[key] = changed.clone()
					}
				}
			}
		} else {
			default.insert(key.into(), changed.clone());
		}
	}
}
