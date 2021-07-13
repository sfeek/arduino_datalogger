#![windows_subsystem = "windows"]
use fltk::{app::*, button::*, dialog::*, frame::*, group::*, input::*, text::*, window::*, prelude::*,};

#[derive(Clone, Debug)]
// Define a struct for the form fields
struct Parameters {
    output: TextDisplay,
    com_port: Input,
    file_name: Input, // Empty space to store the filename on form
}

fn main() {
    let app = App::default();

    // Main Window
    let mut wind = Window::new(
        100,
        100,
        420,
        530,
        "Serial Port Data Logger v1.0",
    );

    // Fill the form structure
    let mut parameters = Parameters {
        output: TextDisplay::new(10, 10, 400, 400, ""),
        com_port: Input::new(230,420,100,30,"COM Port"),
        file_name: Input::new(0,0,0,0,""), // Empty space to store the filename on form
    };

    // Text buffers for our inputs and output
    let buf_out = TextBuffer::default();

    // Set output buffer
    parameters.output.set_buffer(Some(buf_out));
    parameters.file_name.hide(); // Hide the filename field

    // Clone the parameters to use for the clear function
    let mut start_parameters = parameters.clone();
    let mut stop_parameters = parameters.clone();
    let mut file_parameters = parameters.clone();

    // Start button
    let mut start_button = Button::new(30, 420, 100, 40, "Start");
    start_button.set_callback(move || start(&mut start_parameters));

    // Stop button
    let mut stop_button = Button::new(30, 470, 100, 40, "Stop");
    stop_button.set_callback(move || stop(&mut stop_parameters));

    // File Choose button
    let mut file_button = Button::new(200, 470, 100, 40, "File");
    file_button.set_callback(move || file(&app, &mut file_parameters));

    // Show the window
    wind.end();
    wind.show();

    // Enter main loop
    app.run().unwrap();
}

fn start(p: &mut Parameters) {
    p.output.buffer().unwrap().set_text(&p.file_name.value());
}

fn stop(p: &mut Parameters) {
    p.output.buffer().unwrap().set_text("stop");
}

fn file(app: &App, p: &mut Parameters) {

    let mut fc = FileChooser::new(".", "csv", FileChooserType::Create, "Choose Output File");
    
    fc.show();
    fc.window().set_pos(300, 300);

    while fc.shown() {
        app.wait();
    }

    // User hit cancel?
    if fc.value(1).is_none() {
        return;
    }

    p.file_name.set_value(&fc.value(1).unwrap());
}
