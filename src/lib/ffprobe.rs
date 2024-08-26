use std::io::Error;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::ffprobe_struct::{FFProbeOutput, Stream};

pub fn ffprobe(input: &Path) -> Result<Vec<Stream>, Error> {
	let ffprobe = Command::new("ffprobe")
		.arg("-i").arg(input.to_str().unwrap())
		.arg("-hide_banner")
		.arg("-loglevel").arg("quiet")
		.arg("-print_format").arg("json")
		.arg("-show_streams")
		.stdout(Stdio::piped())
		.spawn().expect("failed to run ffprobe");

	let child_output = ffprobe.wait_with_output().expect("failed to wait for ffprobe");
	let stdout = String::from_utf8(child_output.stdout).expect("stdout contained corrupted data");

	let output: FFProbeOutput = serde_json::from_str(stdout.as_str()).expect("failed to deserialize ffprobe output");

	Ok(output.streams)
}