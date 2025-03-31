use crate::commands::InfoArgs;
use crate::common::ffprobe_output;
use crate::vec_push_ext::PushStrExt;
use anyhow::Result;
use colored::{Color, Colorize};
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType;

pub(crate) fn ffmpeg_info(args: &InfoArgs) -> Result<()> {
	let probe = ffprobe_output(&args.input)?;

	if probe.get_first_video_stream().is_none() {
		eprintln!("NOTE: The input file has no video streams!")
	}
	if probe.get_first_audio_stream().is_none() {
		eprintln!("NOTE: The input file has no audio streams!")
	}

	let mut stream_type_index = 0;
	let mut last_codec_type: Option<&StreamType> = None;
	for stream in &probe.streams {
		let index = &stream.index;
		let codec_type = &stream.codec_type;

		if last_codec_type != Some(codec_type) {
			last_codec_type = Some(codec_type);
			stream_type_index = 0
		} else {
			stream_type_index += 1
		}

		let language = stream.tags.as_ref().and_then(|t| t.language.as_ref());
		let title = stream.tags.as_ref().and_then(|t| t.title.as_ref());
		let default = stream.disposition.as_ref().map(|d| d.default).unwrap_or(0) == 1;

		let type_color = {
			match codec_type {
				StreamType::Video => Color::Blue,
				StreamType::Audio => Color::Red,
				StreamType::Subtitle => Color::Magenta,
				StreamType::Data => Color::Green,
			}
		};
		print!("[{}|{}] {}", index, stream_type_index, codec_type.to_string().color(type_color));

		let mut extra_info: Vec<String> = Vec::new();
		if let Some(language) = language {
			extra_info.add(language)
		}
		if let Some(title) = title {
			extra_info.add(format!("\"{title}\""))
		}
		if default {
			extra_info.add("default")
		}
		if !extra_info.is_empty() {
			print!("({})", extra_info.join(", "))
		}

		print!(": ");

		match codec_type {
			StreamType::Video => {
				let codec_name = stream.codec_name.as_ref().unwrap();
				let codec_profile = stream.profile.as_ref().unwrap();
				let pix_fmt = stream.pix_fmt.as_ref().unwrap();

				print!("{codec_name} ({codec_profile}), {pix_fmt} ");

				let width = stream.width.unwrap_or(0);
				let height = stream.height.unwrap_or(0);
				let fps = stream.frame_rate().unwrap_or(0_f64);

				let mut format_info: Vec<String> = Vec::new();
				if let Some(field_order) = &stream.field_order {
					format_info.add(field_order)
				}
				if let Some(color_range) = &stream.color_range {
					format_info.add(color_range)
				}

				let mut color_info: Vec<String> = Vec::new();
				if let Some(color_space) = &stream.color_space {
					color_info.add(color_space)
				}
				if let Some(color_primaries) = &stream.color_primaries {
					color_info.add(color_primaries)
				}
				if let Some(color_transfer) = &stream.color_transfer {
					color_info.add(color_transfer)
				}
				if !color_info.is_empty() {
					// if color space, primaries, and transfer are the same,
					// only print one of them instead of repeating it three times
					if color_info.iter().all(|c| c == &color_info[0]) {
						format_info.add(&color_info[0]);
					} else {
						format_info.add(color_info.join("/"));
					}
				}
				if !format_info.is_empty() {
					print!("({})", format_info.join(", "))
				}

				let fps = format!("{fps:.3}");
				let fps = fps.trim_end_matches("0").trim_end_matches(".");
				if let (Some(sar), Some(dar)) = (&stream.sar, &stream.dar) {
					println!(", {width}×{height} ({sar}/{dar}), {fps} fps")
				} else {
					println!(", {width}×{height}, {fps} fps")
				}
			}
			StreamType::Audio => {
				let codec_name = stream.codec_name.as_ref().unwrap();
				let sample_rate = stream.sample_rate.as_ref().unwrap();
				let channels = stream.channels.unwrap_or(0);
				let sample_fmt = stream.sample_fmt.as_ref().unwrap();

				print!("{codec_name}");
				if let Some(codec_profile) = &stream.profile {
					print!(" ({codec_profile})");
				}

				if let Some(channel_layout) = stream.channel_layout.as_ref() {
					print!(", {sample_rate} Hz, {channels}ch: {channel_layout}, {sample_fmt}");
				} else {
					print!(", {sample_rate} Hz, {channels}ch, {sample_fmt}");
				}

				if let Some(bits_per_sample) = &stream.bits_per_raw_sample {
					print!(" ({bits_per_sample})");
				}

				if let Some(bit_rate) = &stream.bit_rate {
					let bitrate = bit_rate.parse::<f64>().unwrap() / 1000.0;
					print!(", {bitrate} kb/s");
				}

				// println!("{codec_name}({codec_profile}), ")
				println!();
			}
			StreamType::Subtitle => {
				let codec_name = stream.codec_name.as_ref().unwrap();
				println!("{codec_name}")
			}
			StreamType::Data => {
				if let Some(codec_type_string) = stream.codec_tag_string.as_ref() {
					print!("{codec_type_string}")
				} else {
					print!("data?")
				}

				if let Some(handler_name) = stream.tags.as_ref().and_then(|tags| tags.handler_name.as_ref()) {
					print!(" ({handler_name})");
				}

				println!();
			},
		}
	}

	Ok(())
}
