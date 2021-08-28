use std::{fs, io, ops};
use toml::value::Value;

pub fn default() -> Value {
	toml::toml! {
		[buffer.newline]
		detect = true
		use_crlf = false

		[pane.linenumbers]
		enable = true
		suffix = "|"
		background = "#282a2e"
		foreground = "#808281"
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
