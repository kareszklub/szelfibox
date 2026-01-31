use log::{error, info};
use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn start_camera_stream(app: AppHandle) {
    std::thread::spawn(move || {
        let fifo_path = "/tmp/scrcpy_stream.fifo";

        // 1. Create the FIFO if it doesn't exist
        // On Linux/macOS we use mkfifo. On Windows, this logic would need adaptation.
        let _ = fs::remove_file(fifo_path); // Clean up old one
        Command::new("mkfifo")
            .arg(fifo_path)
            .status()
            .expect("Failed to create FIFO");

        info!("Created FIFO at {}", fifo_path);

        // 2. Start Scrcpy - Writing to the FIFO
        let mut scrcpy_proc = Command::new("scrcpy")
            .args(&[
                "--video-source=camera",
                "--camera-size=1280x960",
                "--no-audio",
                "--no-window",
                "--record",
                fifo_path,
                "--record-format=mkv",
                "--video-bit-rate=4M",
                "--max-fps=30",
                // Add these for latency:
                "--video-codec=h264",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn scrcpy");

        // 3. Start FFmpeg - Reading from the FIFO
        let mut ffmpeg_proc = Command::new("ffmpeg")
            .args(&[
                "-probesize",
                "32", // Don't wait to analyze the stream
                "-analyzeduration",
                "0", // Start processing immediately
                "-i",
                fifo_path,
                "-c:v",
                "copy",
                "-f",
                "h264",
                "-bsf:v",
                "h264_mp4toannexb",
                "-tune",
                "zerolatency", // Crucial for real-time
                "-fflags",
                "nobuffer+flush_packets", // Combined flags correctly
                "-flags",
                "low_delay", // Force low delay mode
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn ffmpeg");

        let mut final_stdout = ffmpeg_proc.stdout.take().unwrap();
        let mut buffer = [0u8; 65536];
        let mut pending_batch = Vec::new();
        let mut nal_count = 0;

        // Adjust this constant to find your sweet spot (usually 3-10)
        const NAL_UNITS_PER_PACKET: usize = 3;

        loop {
            match final_stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = &buffer[..n];

                    for &byte in chunk {
                        pending_batch.push(byte);

                        let len = pending_batch.len();
                        // Check for NAL start code: 0x00 00 00 01
                        if len >= 4 && &pending_batch[len - 4..] == [0, 0, 0, 1] {
                            // We found a start code, but it marks the START of a new unit.
                            // The unit that just finished is everything before these 4 bytes.
                            nal_count += 1;

                            if nal_count >= NAL_UNITS_PER_PACKET {
                                // Extract the completed units (everything except the new start code)
                                let to_send = pending_batch[..len - 4].to_vec();

                                if !to_send.is_empty() {
                                    // info!("Sending batch of {} NAL units", nal_count);
                                    let _ = app.emit("video-packet", to_send);
                                }

                                // Reset for the next batch, keeping the start code we just found
                                pending_batch = vec![0, 0, 0, 1];
                                nal_count = 0;
                            }
                        }
                    }

                    // Safety: prevent memory bloat if we don't hit the NAL count quickly
                    if pending_batch.len() > 1024 * 512 {
                        // 512KB
                        let _ = app.emit("video-packet", pending_batch.clone());
                        pending_batch.clear();
                        nal_count = 0;
                    }
                }
                Err(_) => break,
            }
        }

        // Cleanup
        let _ = scrcpy_proc.kill();
        let _ = ffmpeg_proc.kill();
        let _ = fs::remove_file(fifo_path);
    });
}
