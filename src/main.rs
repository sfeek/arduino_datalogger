#![windows_subsystem = "windows"]
use fltk::{app::*, button::*, dialog::*, input::*, misc::*, text::*, window::*};
use serialport::{available_ports, SerialPortType};
use std::io::prelude::*;
use std::{fs::File, io::BufReader, io::Write, io::self, sync::Arc, sync::RwLock, thread, time};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Start,
    Stop,
    File,
}

fn main() {
    // Thread Status Variable with R/W Locks
    let running = Arc::new(RwLock::new(0));

    // Get app handle
    let app = App::default();

    // Place to put the filename
    let mut file_name: String = String::new();

    // Main Window
    let mut wind = Window::new(100, 100, 420, 530, "Serial Port Data Logger v1.0");

    // Output and Com Port text boxes
    let mut output: TextDisplay = TextDisplay::new(10, 10, 400, 400, "");
    let mut com_port: InputChoice = InputChoice::new(220, 420, 80, 30, "COM Port");
    let mut com_settings: IntInput = IntInput::new(220, 470, 70, 30, "COM Baud");
    com_settings.set_value("9600");
    let buf_out = TextBuffer::default();
    output.set_buffer(Some(buf_out));

    // Look for usable COM ports and populate drop down
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        com_port.add(&p.port_name);
    }

    // Define Buttons
    let mut start_button = Button::new(30, 420, 100, 40, "Start");
    let mut stop_button = Button::new(30, 470, 100, 40, "Stop");
    let mut file_button = Button::new(310, 470, 100, 40, "File");

    // Make sure Stop button is grayed out initially
    stop_button.deactivate();

    // Show the window
    wind.end();
    wind.show();

    // Setup the message handler
    let (s, r) = channel::<Message>();

    // Attach messages to event emitters
    start_button.emit(s, Message::Start);
    stop_button.emit(s, Message::Stop);
    file_button.emit(s, Message::File);

    // Main Message Loop
    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::Start => start(
                    &app,
                    &running,
                    &mut com_port,
                    &mut com_settings,
                    &file_name,
                    &mut output,
                    &mut start_button,
                    &mut stop_button,
                ),
                Message::Stop => stop(&running, &mut start_button, &mut stop_button),
                Message::File => file_name = file_chooser(&app),
            }
        }
    }
}

// Start logging to CSV
fn start(
    app: &App,
    running: &Arc<RwLock<i32>>,
    com_port: &mut InputChoice,
    com_settings: &mut IntInput,
    file_name: &String,
    output: &mut TextDisplay,
    start_button: &mut Button,
    stop_button: &mut Button,
) {
    // Toggle the start/stop buttons
    start_button.deactivate();
    stop_button.activate();

    // Set thread status to running
    *running.write().unwrap() = 1;

    // Make a clone of the thread status for the sub thread
    let thread_status = Arc::clone(&running);

    // Get a clone the form controls
    let out_handle = output.clone();
    let baud = com_settings.value().parse::<u32>().unwrap();
    
    let port = match com_port.value() {
        Some(val) => {format!("{}",val)},
        None => {String::from("")},
    };

    // Spawn the subthread to take readings
    thread::spawn(move || {
        // Open the serial port
        let serial_port = serialport::new(port, baud)
            .timeout(time::Duration::from_millis(10000))
            .open();

        // Write stuff to the output window
        match serial_port {
            Ok(mut serial_port) => {
                let mut serial_buf: Vec<u8> = vec![0; 1];
                let mut out_buf: Vec<u8> = Vec::new();
                loop {
                    // If the thread status changes to stopped, leave the thread
                    if *thread_status.read().unwrap() == 0 {
                        out_handle.buffer().unwrap().append("\n");
                        break;
                    }
                    match serial_port.read(serial_buf.as_mut_slice()) {
                        Ok(_t) => {
                            out_buf.push(serial_buf[0]);
                            if serial_buf[0] == 13 {
                                out_handle.buffer().unwrap().append(std::str::from_utf8(&out_buf).unwrap());
                                out_buf.clear();
                            }
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(ref e) => {},
                    }
                }
            }
            Err(ref e) => {
            }
        }
    });
}

// Stop logging
fn stop(running: &Arc<RwLock<i32>>, start_button: &mut Button, stop_button: &mut Button) {
    // Toggle the start/stop buttons
    start_button.activate();
    stop_button.deactivate();

    // Set thread status to not running
    *running.write().unwrap() = 0;
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
