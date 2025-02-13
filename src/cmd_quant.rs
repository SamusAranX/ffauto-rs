use anyhow::Result;

use ffauto_rs::ffmpeg::ffmpeg::ffmpeg;
use ffauto_rs::ffmpeg::ffprobe::ffprobe;

use crate::commands::QuantArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_quant(args: &QuantArgs, debug: bool) -> Result<()> {
	let probe = ffprobe(&args.input, false)?;

	let first_video_stream = probe.get_first_video_stream();
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	let seek = args.parse_seek();
	if let Some(seek) = seek {
		ffmpeg_args.add_two("-ss", format!("{}", seek.as_secs_f64()));
	}

	// add input -t argument to ensure ffmpeg only reads one frame
	ffmpeg_args.add("-t");
	if let Some(fps) = video_stream.frame_rate() {
		// if we know the input video's frame rate, we can accurately limit the number of read frames to just one
		ffmpeg_args.push(format!("{}", 1.0 / fps));
	} else {
		// else we just say "take the first second's worth of frames" and hope for the best
		ffmpeg_args.add("1");
	}

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);

	ffmpeg_args.add("-an");
	ffmpeg_args.add("-dn");
	ffmpeg_args.add("-sn");
	ffmpeg_args.add_two("-frames:v", "1");
	ffmpeg_args.add_two("-update", "1");

	// region Video Filtering

	let mut video_filter: Vec<String> = vec!();
	video_filter.add("select=eq(n\\,0)");

	if let Some(crop_filter) = args.generate_crop_filter() {
		video_filter.push(crop_filter);
	}

	if let Some(scale_filter) = args.generate_scale_filter() {
		video_filter.push(scale_filter);
	}

	if video_stream.is_hdr() {
		video_filter.push(TONEMAP_FILTER.parse()?);
	}

	if let Some(color_filters) = args.generate_color_filters() {
		video_filter.push(color_filters);
	}

	let video_filter_str = video_filter.join(",");
	let palette_filters = args.generate_palette_filters()?;
	ffmpeg_args.add_two("-filter_complex", format!("{video_filter_str}{palette_filters}"));

	// endregion

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	ffmpeg(&ffmpeg_args, false, debug)
}
