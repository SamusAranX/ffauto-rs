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

pub fn ffmpeg(in_args: &[String], show_progress: bool, debug: bool) -> Result<()> {
	let temp_file = Builder::new()
		.prefix("ffmpeg")
		.suffix(".txt")
		.tempfile()
		.context("Couldn't create temp file: {e}")?;

	let mut args = vec![
		String::from("-progress"),
		temp_file.path().to_str().unwrap().to_string()
	];
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
							frames_processed = Some(frame.trim().parse::<u64>().unwrap_or_default());
						}
						("fps", fps) => {
							frames_per_second = Some(fps.trim().parse::<f64>().unwrap_or_default());
						}
						("out_time", time) => {
							out_time = parse_ffmpeg_duration(time);
						}
						("bitrate", bitrate) => {
							encode_bitrate = Some(bitrate.trim().to_string());
						}
						("speed", speed) => {
							encode_speed = Some(speed.trim().trim_end_matches('x').parse::<f32>().unwrap_or_default());
						}
						("progress", "continue") => (),
						("progress", "end") => {
							#[cfg(debug_assertions)]
							eprintln!("PROCESSING HAS ENDED");
							break;
						}
						("stream_0_0_q" | "total_size" | "out_time_us" | "out_time_ms" | "dup_frames" | "drop_frames", _) => (),
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

			if let (Some(frame), Some(fps), Some(time), Some(bitrate), Some(speed)) = (frames_processed, frames_per_second, out_time, &encode_bitrate, encode_speed) {
				println!(
					"frame: {frame} - fps: {fps:.2} - time: {} - bitrate: {bitrate} - speed: {speed:.3}x",
					format_ffmpeg_timestamp(time, TwoDigits)
				);

				frames_processed = None;
				frames_per_second = None;
				out_time = None;
				encode_bitrate = None;
				encode_speed = None;
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
