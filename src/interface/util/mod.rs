use crossterm::style;
use colors_transform::{Color, Rgb};
use toml::value;

pub fn color_guess(color: &str) -> Option<style::Color> {
	if let Ok(rgb) = Rgb::from_hex_str(color) {
		let (r, g, b) = (
			rgb.get_red() as u8,
			rgb.get_green() as u8,
			rgb.get_blue() as u8
		);

		Some(style::Color::Rgb {r, g, b})
	} else {
		style::Color::parse_ansi(color)
	}
}

pub fn colors_guess(foreground: &str, background: &str) -> style::Colors {
	style::Colors {
		foreground: color_guess(foreground),
		background: color_guess(background)
	}
}

pub fn attributes_load(options: &value::Value) -> style::Attributes {
	use style::{Attribute, Attributes};

	let mut attributes = Vec::new();

	for (key, value) in options.as_table().unwrap() {
		if value.as_bool().unwrap() {
			match key.as_str() {
				"bold" => attributes.push(Attribute::Bold),
				"italic" => attributes.push(Attribute::Italic),
				"dim" => attributes.push(Attribute::Dim),
				_ => {}
			}
		}
	}

	Attributes::from(attributes.as_slice())
}