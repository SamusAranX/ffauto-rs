use crate::ffmpeg::timestamps::TimestampFormat::TwoDigits;
use crate::ffmpeg::timestamps::{format_ffmpeg_timestamp, parse_ffmpeg_duration};
use anyhow::{Context, Result};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tempfile::Builder;

pub fn ffmpeg(in_args: &[String], accelerator: Option<String>, show_progress: bool, debug: bool) -> Result<()> {
	let temp_file = Builder::new()
		.prefix("ffmpeg")
		.suffix(".txt")
		.tempfile()
		.context("Couldn't create temp file: {e}")?;

	let mut args = vec![
		"-progress".to_string(),
		temp_file.path().to_str().unwrap().to_string(),
	];

	if let Some(accelerator) = accelerator {
		args.extend(["-hwaccel".to_string(), accelerator]);
	}

	args.extend(in_args.to_vec());

	if debug {
		println!("{:#^40}", " DEBUG MODE ");

		let ffmpeg_args = &args
			.iter()
			.map(|a| if a.contains(" ") { format!("\"{a}\"") } else { a.to_string() })
			.collect::<Vec<String>>();

		println!("full command: ffmpeg {}", ffmpeg_args.join(" "));
		let mut stdout = io::stdout();
		let stdin = io::stdin();
		write!(stdout, "{:#^40}", " Press Enter to continue… ").unwrap();
		stdout.flush().unwrap();
		let _ = stdin.read_line(&mut "".to_string()).unwrap();
		writeln!(stdout, "Continuing…").unwrap();
	}

	let mut ffmpeg = Command::new("ffmpeg");
	let ffmpeg = ffmpeg.args(args);

	let start = Instant::now();

	#[allow(clippy::zombie_processes)]
	let mut process = ffmpeg.spawn().expect("failed to run ffmpeg");

	if show_progress {
		let progress_file = File::open(temp_file.path())?;
		let mut reader = BufReader::new(progress_file);

		let mut pos = 0;
		let mut last_progress = Instant::now();

		let mut frames_processed = None;
		let mut frames_per_second = None;
		let mut out_time = None;
		let mut encode_bitrate = None;
		let mut encode_speed = None;
		let mut total_size = None;

		loop {
			let mut line = String::new();
			let res = reader.read_line(&mut line);
			match res {
				Ok(0) => {
					// stats_period defaults to 0.5 seconds, but sometimes heavy processing means output gets delayed
					// within this window, assume that an EOF just means ffmpeg needs more time
					if last_progress.elapsed() < Duration::from_secs(5) {
						sleep(Duration::from_millis(200));

						reader.seek(SeekFrom::Start(pos)).context("Failed to seek to resume point")?;

						continue;
					}

					break;
				}
				Ok(len) => {
					last_progress = Instant::now();
					pos += len as u64;

					let (key, value) = match line.trim().split_once('=') {
						Some(v) => v,
						None => break,
					};

					match (key, value) {
						("frame", frame) => {
							frames_processed = frame.trim().parse::<u64>().ok();
						}
						("fps", fps) => {
							frames_per_second = fps.trim().parse::<f64>().ok();
						}
						("out_time", time) => {
							out_time = parse_ffmpeg_duration(time.trim());
						}
						("bitrate", bitrate) => {
							encode_bitrate = Some(bitrate.trim().to_string());
						}
						("speed", speed) => {
							encode_speed = speed.trim().trim_end_matches('x').parse::<f64>().ok();
						}
						("total_size", size) => {
							total_size = size.trim().parse::<u64>().ok()
						}
						("progress", "continue") => (),
						("progress", "end") => {
							#[cfg(debug_assertions)]
							eprintln!("PROCESSING HAS ENDED");
							break;
						}
						("out_time_us" | "out_time_ms" | "dup_frames" | "drop_frames", _) => (),
						_ if key.starts_with("stream_") && key.ends_with("_q") => (),
						_ => {
							#[cfg(debug_assertions)]
							eprintln!("Unknown progress value: {key} = {value}");
						}
					}
				}
				Err(e) => {
					eprintln!("failed to read progress file: {e:?}");
					break;
				}
			}

			// values for keys "bitrate", "out_time", and "speed" can be N/A for a while after starting an encode
			// so we process those values separately
			// there's probably a better way of doing this but I'm lazy and this works
			if let (Some(frame), Some(fps), Some(size)) = (frames_processed, frames_per_second, total_size) {
				let timestamp = out_time
					.map(|time| format_ffmpeg_timestamp(time, TwoDigits))
					.unwrap_or("N/A".to_string());
				let bitrate = encode_bitrate
					.unwrap_or("N/A".to_string());
				let speed = encode_speed
					.map(|speed| format!("{speed:.3}x"))
					.unwrap_or("N/A".to_string());
				
				let formatted_size = {
					if cfg!(target_os = "macos") {
						humansize::format_size(size, humansize::DECIMAL)
					} else {
						humansize::format_size(size, humansize::WINDOWS)
					}
				};

				println!("frame: {frame} - fps: {fps:.2} - time: {timestamp} - size: {formatted_size} - bitrate: {bitrate} - speed: {speed}");

				frames_processed = None;
				frames_per_second = None;
				out_time = None;
				encode_bitrate = None;
				encode_speed = None;
				total_size = None;
			}
		}
	}

	let exit_status = process.wait().expect("failed to wait for ffmpeg");
	if !exit_status.success() {
		anyhow::bail!("ffmpeg exited with status code {}", exit_status.code().unwrap_or(-1))
	}

	let execution_time = start.elapsed();
	println!("Encoding took {:.2}s!", execution_time.as_secs_f64());

	Ok(())
}
