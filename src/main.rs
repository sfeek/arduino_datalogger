#![windows_subsystem = "windows"]
use chrono::prelude::*;
use fltk::{app::*, button::*, dialog::*, misc::*, prelude::*, text::*, window::*};
use std::io::prelude::*;
use std::{fs::OpenOptions, io::Write, sync::Arc, sync::RwLock, thread};

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
    let mut wind = Window::new(100, 100, 800, 530, "Serial Port Data Logger v1.0");

    // Output and Com Port text boxes
    let mut output: SimpleTerminal = SimpleTerminal::new(10, 10, 780, 400, "");
    let mut com_port: InputChoice = InputChoice::new(350, 420, 80, 30, "COM Port");
    let mut com_settings: InputChoice = InputChoice::new(350, 470, 80, 30, "COM Baud");

    output.set_stay_at_bottom(true);
    output.set_ansi(true);
    output.set_cursor_style(Cursor::Simple);

    let bauds: Vec<&str> = vec!["1200", "9600", "19200", "115200"];

    for b in bauds {
        com_settings.add(b);
    }

    // Look for usable COM ports and populate drop down
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        com_port.add(&p.port_name);
    }

    // Define Buttons
    let mut start_button = Button::new(30, 420, 100, 40, "Start");
    let mut stop_button = Button::new(30, 470, 100, 40, "Stop");
    let mut file_button = Button::new(150, 470, 100, 40, "File");

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
#[allow(clippy::ptr_arg)]
fn start(
    running: &Arc<RwLock<i32>>,
    com_port: &mut InputChoice,
    com_settings: &mut InputChoice,
    file_name: &String,
    output: &mut SimpleTerminal,
    start_button: &mut Button,
    stop_button: &mut Button,
) {
    // Make sure user has choosen a file
    if file_name.is_empty() {
        return;
    }
    // Toggle the start/stop buttons
    start_button.deactivate();
    stop_button.activate();

    // Set thread status to running
    *running.write().unwrap() = 1;

    // Make a clone of the thread status for the sub thread
    let thread_status = Arc::clone(running);

    // Get a clone the form controls
    let mut out_handle = output.clone();
    let file_name = file_name.clone();
    let mut start_button = start_button.clone();
    let mut stop_button = stop_button.clone();

    // Get settings for the COM port
    let (baud, port) = match (com_settings.value(), com_port.value()) {
        (Some(val), Some(port)) => (val.parse::<u32>().unwrap(), port),
        (_, _) => return,
    };

    // Spawn the subthread to take readings
    thread::spawn(move || {
        // Buffers etc.
        let mut serial_buf: Vec<u8> = vec![0; 1];
        let mut out_buf: Vec<u8> = Vec::new();
        let mut final_buf: Vec<u8> = Vec::new();

        // Open the serial port
        let mut serial_port = match serialport::new(port, baud).open() {
            Ok(v) => v,
            Err(_) => {
                out_handle.append("Serial Port Open Error");
                *thread_status.write().unwrap() = 0;
                return;
            }
        };

        // Open the file
        let mut f = match OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_name)
        {
            Ok(v) => v,
            Err(_) => {
                out_handle.append("File Open Error");
                *thread_status.write().unwrap() = 0;
                return;
            }
        };

        // Read data and write to window and file
        // Main Loop to read bytes from the serial port and record them
        loop {
            // If the thread status changes to stopped, leave the thread and reset the buttons
            if *thread_status.read().unwrap() == 0 {
                start_button.activate();
                stop_button.deactivate();
                break;
            }

            // Read byte from the port
            if serial_port.read(serial_buf.as_mut_slice()).is_ok() {
                match serial_buf[0] {
                    // reached end of line, record and display data
                    13 => {
                        // Get timestamp
                        let mut time_stamp: Vec<u8> = Local::now()
                            .format("%Y-%m-%d,%H:%M:%S,")
                            .to_string()
                            .into_bytes();

                        // Append time stamp and line of data
                        final_buf.append(&mut time_stamp);
                        final_buf.append(&mut out_buf);
                        final_buf.append(&mut "\n".to_string().into_bytes());

                        // Send to display window
                        out_handle.append(std::str::from_utf8(&final_buf).unwrap());

                        // Send to file
                        if f.write_all(&final_buf).is_err() {
                            *thread_status.write().unwrap() = 0;
                        }

                        // Clear out buffers for the next line
                        out_buf.clear();
                        final_buf.clear();
                    }
                    // Throw away line feeds
                    10 => {}
                    // Keep everything else
                    _ => out_buf.push(serial_buf[0]),
                }
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
    let mut fc = FileChooser::new(".", "*.csv", FileChooserType::Create, "Choose Output File");

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
