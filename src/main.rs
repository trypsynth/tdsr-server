#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![windows_subsystem = "windows"]

use std::{
	env,
	io::{BufRead, BufReader},
	net::{TcpListener, TcpStream},
	process, thread,
};

use native_dialog::{DialogBuilder, MessageLevel};
use prismer::{Backend, Prism};
use tao::{
	event::Event,
	event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
	TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem},
};

#[derive(Debug)]
enum UserEvent {
	MenuEvent(tray_icon::menu::MenuEvent),
}

const DEFAULT_PORT: u16 = 64111;
const MAX_LINE_LENGTH: usize = 10000;

fn main() {
	let port = parse_port_from_args().unwrap_or(DEFAULT_PORT);
	let listener = match TcpListener::bind(("0.0.0.0", port)) {
		Ok(listener) => listener,
		Err(e) => {
			show_error(&format!("Unable to bind to port {port}: {e}"));
			return;
		}
	};
	thread::spawn(move || {
		for connection in listener.incoming() {
			thread::spawn(move || match connection {
				Ok(stream) => {
					if let Err(e) = handle_connection(stream) {
						show_error(&format!("Connection error: {e}"));
					}
				}
				Err(e) => show_error(&format!("Failed to accept connection: {e}")),
			});
		}
	});
	run_tray_application();
}

fn parse_port_from_args() -> Option<u16> {
	env::args().nth(1)?.parse().ok()
}

fn run_tray_application() {
	let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
	let proxy = event_loop.create_proxy();
	MenuEvent::set_event_handler(Some(move |event| {
		let _ = proxy.send_event(UserEvent::MenuEvent(event));
	}));
	let mut tray_icon = None;
	let tray_menu = Menu::new();
	let quit_item = MenuItem::new("&Quit", true, None);
	if let Err(e) = tray_menu.append(&quit_item) {
		show_error(&format!("Failed to build tray menu: {e:?}"));
		return;
	}
	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;
		match event {
			Event::NewEvents(tao::event::StartCause::Init) => {
				tray_icon = match TrayIconBuilder::new()
					.with_menu(Box::new(tray_menu.clone()))
					.with_tooltip("TDSR Server")
					.build()
				{
					Ok(icon) => Some(icon),
					Err(e) => {
						show_error(&format!("Tray icon error: {e:?}"));
						process::exit(1);
					}
				};
			}
			Event::UserEvent(UserEvent::MenuEvent(event)) if event.id == quit_item.id() => {
				if let Some(icon) = tray_icon.take() {
					drop(icon);
				}
				*control_flow = ControlFlow::Exit;
			}
			_ => {}
		}
	});
}

fn handle_connection(connection: TcpStream) -> Result<(), String> {
	let mut reader = BufReader::new(connection);
	let mut line = String::new();
	let prism = Prism::new().map_err(|e| format!("Failed to initialize speech: {e}"))?;
	// The best backend is the running screen reader, or a speech engine if there isn't one.
	let backend = prism.create_best().map_err(|e| format!("No speech output is available: {e}"))?;
	while reader.read_line(&mut line).map_err(|e| format!("Failed to read line: {e}"))? > 0 {
		let trimmed_line = line.trim_end_matches(['\n', '\r']);
		if let Some((command, arg)) = trimmed_line.split_at_checked(1) {
			process_command(command, arg, &backend);
		}
		line.clear();
	}
	Ok(())
}

fn process_command(command: &str, arg: &str, backend: &Backend<'_>) {
	match command {
		"s" | "l" if !arg.is_empty() => {
			let cleaned_text = arg.replace('\u{23CE}', " ");
			speak_in_chunks(&cleaned_text, backend);
		}
		"x" => stop_speaking(backend),
		_ => {}
	}
}

fn speak_in_chunks(text: &str, backend: &Backend<'_>) {
	let mut chunk = String::new();
	for word in text.split_whitespace() {
		if chunk.is_empty() {
			chunk.push_str(word);
			continue;
		}
		let next_len = chunk.len() + 1 + word.len();
		if next_len > MAX_LINE_LENGTH {
			speak(&chunk, backend);
			chunk.clear();
		} else {
			chunk.push(' ');
		}
		chunk.push_str(word);
	}
	if !chunk.is_empty() {
		speak(&chunk, backend);
	}
}

fn speak(text: &str, backend: &Backend<'_>) {
	if let Err(e) = backend.speak(text, false) {
		show_error(&format!("Failed to speak: {e}"));
	}
}

fn stop_speaking(backend: &Backend<'_>) {
	if let Err(e) = backend.stop() {
		show_error(&format!("Failed to stop speaking: {e}"));
	}
}

fn show_error(message: &str) {
	let _ = DialogBuilder::message()
		.set_title("TDSR Server Error")
		.set_level(MessageLevel::Error)
		.set_text(message)
		.alert();
}
