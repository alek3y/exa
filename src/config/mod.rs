use toml::value::Value;

pub fn default() -> Value {
	toml::toml! {
		[buffer]
		tab_size = 3

		[buffer.newline]
		detect = true
		use_crlf = false

		[pane]
		background = "2;29;31;33"
		foreground = "2;197;200;198"

		[pane.linenumbers]
		enable = true
		suffix = "|"
		foreground = "2;138;140;139"
		background = "2;40;42;46"
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
