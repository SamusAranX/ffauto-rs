use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use anyhow::Result;

use ffauto_rs::ffmpeg::ffprobe::ffprobe;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::Video;

use crate::commands::{Cli, GIFArgs};
use crate::common::{add_crop_scale_tonemap_filters, add_color_sharpness_filters, debug_pause, generate_palette_filtergraph, parse_duration, parse_seek};
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_gif(cli: &Cli, args: &GIFArgs) -> Result<()> {
	let probe = ffprobe(&args.input, false)?;

	let first_video_stream = probe.streams.iter().find(|s| s.codec_type == Video);
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let video_duration = probe.duration()
		.inspect_err(|e| eprintln!("{e}"))?;

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

	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);

	ffmpeg_args.add("-an");
	ffmpeg_args.add("-dn");
	ffmpeg_args.add("-sn");

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

	// region Video Filtering

	let mut video_filter: Vec<String> = vec![];

	if let Some(fps) = args.framerate {
		video_filter.push(format!("fps=fps={fps:.3}"));
	} else if let (Some(fps_mult), Some(fps)) = (args.framerate_mult, video_stream.frame_rate()) {
		video_filter.push(format!("fps=fps={:.3}", fps * fps_mult));
	}

	add_crop_scale_tonemap_filters(&mut video_filter, cli, video_stream.color_transfer.unwrap_or_default())?;
	add_color_sharpness_filters(&mut video_filter, args.brightness, args.contrast, args.saturation, args.sharpness);

	if fade_in > 0.0 {
		video_filter.push(format!("fade=t=in:st=0:d={fade_in:.3}"));
	}
	if fade_out > 0.0 {
		video_filter.push(format!("fade=t=out:st={fade_out_start:.3}:d={fade_out:.3}"));
	}

	let video_filter_str = video_filter.join(",");
	let filter_complex = generate_palette_filtergraph(
		true, args.dedup,
		video_filter_str,
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

	if cli.debug {
		debug_pause(args, &ffmpeg_args);
	}

	let start = Instant::now();

	let mut ffmpeg = Command::new("ffmpeg")
		.args(ffmpeg_args)
		.spawn().expect("failed to run ffmpeg");

	let exit_status = ffmpeg.wait().expect("failed to wait for ffmpeg");
	if !exit_status.success() {
		return Err(anyhow!("ffmpeg exited with status code {}", exit_status.code().unwrap_or(-1)));
	}

	let execution_time = start.elapsed();
	println!("Encoding took {:.2}s!", execution_time.as_secs_f64());

	Ok(())
}