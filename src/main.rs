use std::{io, time::Duration};
use crossterm::*;
use exa::config::Config;

const REFRESH: u64 = 100;

fn main() {
	let config = dirs::config_dir().map(|mut config_dir| {
		config_dir.push("exa/config.toml");
		let config_dir = config_dir.as_path();

		let mut loaded_config = None;
		if let Some(config_dir) = config_dir.to_str() {
			loaded_config = Config::read(config_dir).ok();
		}

		loaded_config.unwrap_or_else(|| Config::from(None))
	}).unwrap();

	let mut stdout = io::stdout();
	terminal::enable_raw_mode().unwrap();
	execute!(stdout, 
		terminal::EnterAlternateScreen,
		terminal::Clear(terminal::ClearType::All)
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

	terminal::disable_raw_mode().unwrap();
}
