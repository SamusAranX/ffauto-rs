use std::time::Duration;

use anyhow::Result;
use ffauto_rs::ffmpeg::ffmpeg::ffmpeg;

use crate::commands::{Cli, GIFArgs};
use crate::common::*;
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_gif(cli: &Cli, args: &GIFArgs) -> Result<()> {
	let probe = ffprobe_output(&args.input)?;

	let first_video_stream = probe.get_first_video_stream();
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let video_duration = probe.duration()?;

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	let seek = parse_seek(&cli.seek);
	let duration = parse_duration(seek, &args.duration, &args.duration_to);

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

	add_fps_filter(&mut video_filter, args.framerate, args.framerate_mult, video_stream.frame_rate());
	add_crop_scale_tonemap_filters(&mut video_filter, cli, video_stream.is_hdr())?;
	add_color_sharpness_filters(&mut video_filter, args.brightness, args.contrast, args.saturation, args.sharpness);

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade != 0.0 {
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

	let filter_complex = generate_palette_filtergraph(
		true, args.dedup,
		video_filter,
		&args.palette_file, &args.palette_name,
		args.num_colors, &args.stats_mode, args.diff_rect,
		&args.dither, args.bayer_scale,
	)?;

	ffmpeg_args.add_two("-filter_complex", filter_complex);

	// endregion

	if args.dedup {
		ffmpeg_args.add_two("-fps_mode", "vfr");
	}
	ffmpeg_args.add_two("-f", "gif");
	ffmpeg_args.add_two("-loop", "0");

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	ffmpeg(&ffmpeg_args, false, cli.debug)
}