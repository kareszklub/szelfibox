use std::io::{self, BufRead, BufReader};
use std::time::Duration;

use tauri::Emitter;

pub fn run_pico_buttons(app: tauri::AppHandle) {
    // Configuration
    let port_name = "/dev/ttyACM0";
    let baud_rate = 115200; // Match this to your device's baud rate

    println!("Attempting to open serial port: {}", port_name);

    // Open the serial port
    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1000)) // 1 second timeout for read operations
        .open();

    match port {
        Ok(port) => {
            println!("Port open. Listening for button presses...");

            // Wrap the port in a BufReader to easily use read_line()
            let mut reader = BufReader::new(port);
            let mut line_buffer = String::new();

            loop {
                // Clear the buffer to prepare for new data
                line_buffer.clear();

                // Attempt to read a line ending in \n
                match reader.read_line(&mut line_buffer) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            // .trim() removes the \n and any \r (common in serial data)
                            match line_buffer.trim() {
                                "1" => app.emit::<i32>("button", 1).unwrap(),
                                "2" => app.emit::<i32>("button", 2).unwrap(),
                                _ => eprintln!(
                                    "Warning: Received unexpected data: {:?}",
                                    line_buffer
                                ),
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                        // A timeout happened (no data for 1 sec).
                        // We just continue the loop and try reading again.
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error reading from port: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            eprintln!(
                "Hint: Check if the device is plugged in or if you have permission (see below)."
            );
        }
    }
}
