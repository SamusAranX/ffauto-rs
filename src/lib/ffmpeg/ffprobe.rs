use std::io::Error;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::ffmpeg::ffprobe_struct::{FFProbeOutput, Stream};

pub fn ffprobe(input: &Path, count_frames: bool) -> Result<Vec<Stream>, Error> {
	let mut ffprobe_args: Vec<String> = vec![
		"-i".to_string(), input.to_str().unwrap().to_string(),
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "quiet".to_string(),
		"-print_format".to_string(), "json".to_string(),
		"-show_streams".to_string()
	];
	if count_frames {
		ffprobe_args.push("-count_frames".to_string());
	}

	let ffprobe = Command::new("ffprobe")
		.args(ffprobe_args)
		.stdout(Stdio::piped())
		.spawn().expect("failed to run ffprobe");

	let child_output = ffprobe.wait_with_output().expect("failed to wait for ffprobe");
	let stdout = String::from_utf8(child_output.stdout).expect("stdout contained corrupted data");

	let output: FFProbeOutput = serde_json::from_str(stdout.as_str()).expect("failed to deserialize ffprobe output");

	Ok(output.streams)
}