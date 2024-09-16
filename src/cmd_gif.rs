use std::process::Command;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::Result;

use ffauto_rs::ffmpeg::ffprobe::ffprobe;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::Video;

use crate::commands::{Cli, GIFArgs};
use crate::common::{add_basic_filters, add_palette_filters, debug_pause, generate_palette_filtergraph, handle_duration, handle_seek};
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_gif(cli: &Cli, args: &GIFArgs) -> Result<()> {
	let probe = ffprobe(&args.input, false)?;

	let first_video_stream = probe.iter().find(|s| s.codec_type == Video);
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	let seek = handle_seek(&mut ffmpeg_args, &args.input, &cli.seek);
	let duration = handle_duration(&mut ffmpeg_args, seek, &args.duration, &args.duration_to);

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade != 0.0 {
		fade_in = args.fade;
		fade_out = args.fade;
	}
	let fade_out_start = duration - fade_out;

	// region Video Filtering

	let mut video_filter: Vec<String> = vec![];

	if let Some(fps) = args.framerate {
		video_filter.push(format!("fps=fps={fps:.3}"));
	}

	add_basic_filters(&mut video_filter, cli, video_stream.color_transfer.unwrap_or_default())?;

	add_palette_filters(&mut video_filter, args.brightness, args.contrast, args.saturation, args.sharpness);

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

	ffmpeg_args.push_str("-filter_complex");
	ffmpeg_args.push(filter_complex);

	// endregion

	if args.dedup {
		ffmpeg_args.push_str("-vsync");
		ffmpeg_args.push_str("0");
	}
	ffmpeg_args.push_str("-f");
	ffmpeg_args.push_str("gif");
	ffmpeg_args.push_str("-loop");
	ffmpeg_args.push_str("0");

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