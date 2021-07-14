#![windows_subsystem = "windows"]
use fltk::{app::*, button::*, dialog::*, input::*, text::*, window::*,};
use std::{io::Write, fs::File,};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Start,
    Stop,
    File,
    Check,
}

// A result can represent either success/ Ok or failure/ Err.
enum Result<T, E> { // T and E are generics. T can contain any type of value, E can be any error.
    Ok(T),
    Err(E),
}

fn main() {
    let mut run: bool = false;
    
    // Get app handle
    let app = App::default();

    // Place to put the filename
    let mut file_name: String = String::new();
    let mut file: File;

    // Main Window
    let mut wind = Window::new(
        100,
        100,
        420,
        530,
        "Serial Port Data Logger v1.0",
    );

    // Output and Com Port text boxes
    let mut output: TextDisplay = TextDisplay::new(10, 10, 400, 400, "");
    let com_port: Input = Input::new(230,420,100,30,"COM Port");
    let buf_out = TextBuffer::default();
    output.set_buffer(Some(buf_out));

    // Buttons
    let mut start_button = Button::new(30, 420, 100, 40, "Start");
    let mut stop_button = Button::new(30, 470, 100, 40, "Stop");
    let mut file_button = Button::new(200, 470, 100, 40, "File");

    stop_button.deactivate();

    // Show the window
    wind.end();
    wind.show();

    // Setup the message handler
    let (s, r) = channel::<Message>();

    start_button.emit(s, Message::Start);
    stop_button.emit(s, Message::Stop);
    file_button.emit(s, Message::File);

    // Main Message Loop
    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::Start => {run = start(&mut start_button, &mut stop_button)},
                Message::Stop => {run = stop(&mut start_button, &mut stop_button)},
                Message::File => {file_name = file_chooser(&app)},
            }
        }
    }
}

// Handle Start Button
fn start(start_button: &mut Button, stop_button: &mut Button) -> bool {
    start_button.deactivate();
    stop_button.activate();
    
    true
}

// Handle Stop Button
fn stop(start_button: &mut Button, stop_button: &mut Button) -> bool {
    start_button.activate();
    stop_button.deactivate();

    false
}

// Handle File Chooser Button
fn file_chooser(app: &App) -> String {
    let mut fc = FileChooser::new(".", "csv", FileChooserType::Create, "Choose Output File");
    
    fc.show();
    fc.window().set_pos(300, 300);

    while fc.shown() {
        app.wait();
    }

    // User hit cancel?
    if fc.value(1).is_none() {
        return String::from("");
    }

    fc.value(1).unwrap()
}
