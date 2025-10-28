use std::time::Duration;

use anyhow::Result;
use ffauto_rs::ffmpeg::ffmpeg::ffmpeg;

use crate::commands::GIFArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_gif(args: &GIFArgs, debug: bool) -> Result<()> {
	let probe = ffprobe_output(&args.input)?;

	let (video_stream, video_stream_id) = probe.checked_get_video_stream_by_index_or_language(&args.video_language, args.video_stream)?;

	let video_duration = probe.duration()?;

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	let seek = args.parse_seek();
	let duration = args.parse_duration();

	if let Some(seek) = seek {
		ffmpeg_args.add_two("-ss", format!("{}", seek.as_secs_f64()));
	}

	// input option to limit the amount of data read
	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);

	// repeat as output option to limit the amount of data written
	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	ffmpeg_args.add("-an");
	ffmpeg_args.add("-dn");
	ffmpeg_args.add("-sn");

	// region Video Filtering

	let mut video_filter: Vec<String> = vec![];

	if let Some(fps_filter) = args.generate_fps_filter(video_stream.frame_rate()) {
		video_filter.push(fps_filter);
	}

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

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade > 0.0 {
		fade_in = args.fade;
		fade_out = args.fade;
	}
	let fade_out_start = if let Some(duration) = duration {
		// duration was given
		duration.as_secs_f64() - fade_out
	} else {
		// duration wasn't given, use video duration
		(video_duration - seek.unwrap_or(Duration::ZERO)).as_secs_f64() - fade_out
	};

	if fade_in > 0.0 {
		video_filter.push(format!("fade=t=in:st=0:d={fade_in:.3}"));
	}
	if fade_out > 0.0 {
		video_filter.push(format!("fade=t=out:st={fade_out_start:.3}:d={fade_out:.3}"));
	}

	let video_filter_str = video_filter.join(",");
	let palette_filters = args.generate_palette_filters()?;
	ffmpeg_args.add_two("-filter_complex", format!("[{video_stream_id}]{video_filter_str}{palette_filters}"));

	// endregion

	if args.dedup {
		ffmpeg_args.add_two("-fps_mode", "vfr");
	}
	ffmpeg_args.add_two("-f", "gif");
	ffmpeg_args.add_two("-loop", "0");

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	ffmpeg(&ffmpeg_args, None, false, debug)
}
