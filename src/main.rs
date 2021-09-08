use std::{io::{self, Write}, fs, time::Duration};
use crossterm::*;
use exa::config;

const REFRESH: u64 = 100;

fn main() {
	let options = dirs::config_dir().map(|mut config_dir| {
		config_dir.push("exa/config.toml");

		let mut loaded_options = config::default();
		if config_dir.exists() {
			let contents = fs::read_to_string(config_dir).ok()
				.and_then(|contents| contents.parse().ok());

			if let Some(custom_options) = contents {
				config::update(&mut loaded_options, &custom_options);
			}
		}

		loaded_options
	}).unwrap();

	let mut stdout = io::stdout();
	terminal::enable_raw_mode().unwrap();
	execute!(stdout, 
		terminal::EnterAlternateScreen,
		terminal::Clear(terminal::ClearType::All),
		cursor::Hide
	).unwrap();

	loop {
		if event::poll(Duration::from_millis(REFRESH)).unwrap() {
			use event::{Event::Key, KeyCode};

			if let Key(key) = event::read().unwrap() {
				match key.code {
					KeyCode::Esc => break,
					_ => ()
				}
			}
		}
	}

	execute!(stdout, cursor::Show).unwrap();
	terminal::disable_raw_mode().unwrap();
}
