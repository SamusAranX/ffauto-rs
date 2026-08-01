use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow};

use crate::ffmpeg::ffprobe_struct::{FFProbeOutput, StreamType};

pub fn ffprobe<P: AsRef<Path>>(input: P, count_frames: bool) -> Result<FFProbeOutput> {
	let mut ffprobe_args = vec![
		"-hide_banner",
		"-loglevel",
		"warning",
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

	let child_output = ffprobe
		.wait_with_output()
		.expect("failed to wait for ffprobe");
	if !child_output.status.success() {
		let stderr = String::from_utf8(child_output.stderr).expect("stderr contained corrupted data");
		anyhow::bail!(stderr.trim().to_string())
	}

	let stdout = String::from_utf8(child_output.stdout).expect("stdout contained corrupted data");

	let mut probe_output = match serde_json::from_str::<FFProbeOutput>(stdout.as_str()) {
		Ok(output) => output,
		Err(e) => {
			// eprintln!("stdout:\n{}", stdout);
			return Err(anyhow!(e));
		}
	};

	// fill in typed stream indices, we're gonna need them later.
	// these count per codec type independently, because stream types can be
	// interleaved in any order (e.g. video, audio, subtitle, audio).
	let mut stream_type_indices: HashMap<StreamType, usize> = HashMap::new();
	for stream in &mut probe_output.streams {
		let stream_type_index = stream_type_indices
			.entry(stream.codec_type.clone())
			.or_insert(0);

		stream.typed_index = *stream_type_index;
		*stream_type_index += 1;
	}

	Ok(probe_output)
}
