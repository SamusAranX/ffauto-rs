use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow};

use crate::ffmpeg::ffprobe_struct::FFProbeOutput;

// ffprobe -hide_banner -loglevel error -print_format json -show_streams -show_format Exclusion\ Zone/mariomovie.mkv
pub fn ffprobe<P: AsRef<Path>>(input: P, count_frames: bool) -> Result<FFProbeOutput> {
	let mut ffprobe_args = vec![
		"-hide_banner",
		"-loglevel",
		"error",
		"-print_format",
		"json",
		"-show_streams",
		"-show_format",
		"-i",
		input.as_ref().to_str().unwrap(),
	];
	if count_frames {
		ffprobe_args.push("-count_frames");
	}

	let ffprobe = Command::new("ffprobe")
		.args(ffprobe_args)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("failed to run ffprobe");

	let child_output = ffprobe.wait_with_output().expect("failed to wait for ffprobe");
	if !child_output.status.success() {
		let stderr = String::from_utf8(child_output.stderr).expect("stderr contained corrupted data");
		anyhow::bail!(stderr.trim().to_string())
	}

	let stdout = String::from_utf8(child_output.stdout).expect("stdout contained corrupted data");

	match serde_json::from_str::<FFProbeOutput>(stdout.as_str()) {
		Ok(output) => Ok(output),
		Err(e) => {
			// eprintln!("stdout:\n{}", stdout);
			Err(anyhow!(e))
		}
	}
}
