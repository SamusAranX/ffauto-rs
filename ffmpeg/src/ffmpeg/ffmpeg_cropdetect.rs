use crate::filters::{Crop, Cropdetect, CropdetectMode, FilterChain, Fps, Metadata, MetadataMode};
use anyhow::bail;
use indexmap::IndexMap;
use std::path::Path;
use std::process::{Command, Stdio};

/// Iterates through the input array and returns either the string that occurs the most or the last string.
fn most_common_or_last(items: impl IntoIterator<Item = impl Into<String>>) -> Option<String> {
	let mut map: IndexMap<String, (usize, usize)> = IndexMap::new();
	for (i, item) in items.into_iter().enumerate() {
		let entry = map.entry(item.into()).or_insert((0, 0));
		entry.0 += 1;
		entry.1 = i;
	}
	map.into_iter()
		.max_by_key(|(_, (count, last_idx))| (*count, *last_idx))
		.map(|(s, _)| s)
}

pub fn ffmpeg_cropdetect<P: AsRef<Path>>(in_file: P) -> anyhow::Result<Crop> {
	let mut args: Vec<String> = vec![
		"-i",
		in_file.as_ref().to_str().unwrap(),
		"-t",
		"10",
		"-f",
		"null",
		"-vf",
	]
	.into_iter()
	.map(Into::into)
	.collect();

	let mut filters = FilterChain::new();
	filters.push(Fps::new(4.0));
	filters.push(Cropdetect::new(CropdetectMode::Black));
	filters.push(Metadata::new(MetadataMode::Print, None));

	args.push(filters.to_string());
	args.push("-".to_string());

	let ffmpeg = Command::new("ffmpeg")
		.args(args)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("failed to run ffmpeg");

	let child_output = ffmpeg
		.wait_with_output()
		.expect("failed to wait for ffmpeg");

	if !child_output.status.success() {
		let stderr = String::from_utf8(child_output.stderr).expect("stderr contained corrupted data");
		bail!(stderr.trim().to_string())
	}

	let stderr = String::from_utf8(child_output.stderr).expect("stdout contained corrupted data");
	let crop_lines = stderr
		.lines()
		.filter_map(|l| {
			if !l.starts_with("[Parsed_cropdetect") {
				return None;
			}

			match l.split_whitespace().last() {
				Some(possibly_crop) => {
					if possibly_crop.starts_with("crop=") {
						return Some(possibly_crop);
					}
					None
				}
				None => None,
			}
		})
		.collect::<Vec<_>>();

	if let Some(crop) = most_common_or_last(crop_lines) {
		return Crop::from_arg(crop);
	}

	bail!("Couldn't determine cropping bounds")
}
