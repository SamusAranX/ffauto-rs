use std::process::Command;
use std::time::Instant;

use anyhow::Result;

use ffauto_rs::ffmpeg::enums::StatsMode;
use ffauto_rs::ffmpeg::ffprobe::ffprobe;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::Video;

use crate::commands::{Cli, QuantArgs};
use crate::common::{add_color_sharpness_filters, add_crop_scale_tonemap_filters, debug_pause, generate_palette_filtergraph, parse_seek};
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_quant(cli: &Cli, args: &QuantArgs) -> Result<()> {
	let probe = ffprobe(&args.input, false)?;

	let first_video_stream = probe.streams.iter().find(|s| s.codec_type == Video);
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	let seek = parse_seek(&cli.seek);
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

	let mut video_filter: Vec<String> = vec![];
	video_filter.add("select=eq(n\\,0)");

	add_crop_scale_tonemap_filters(&mut video_filter, cli, video_stream.color_transfer.unwrap_or_default())?;
	add_color_sharpness_filters(&mut video_filter, args.brightness, args.contrast, args.saturation, args.sharpness);

	let video_filter_str = video_filter.join(",");
	let filter_complex = generate_palette_filtergraph(
		true, false,
		video_filter_str,
		&args.palette_file, &args.palette_name,
		args.num_colors, &StatsMode::default(), false,
		&args.dither, args.bayer_scale,
	)?;

	ffmpeg_args.add_two("-filter_complex", filter_complex);

	// endregion

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
		anyhow::bail!("ffmpeg exited with status code {}", exit_status.code().unwrap_or(-1))
	}

	let execution_time = start.elapsed();
	println!("Encoding took {:.2}s!", execution_time.as_secs_f64());

	Ok(())
}