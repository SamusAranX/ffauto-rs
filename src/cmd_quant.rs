use std::process::Command;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::Result;

use ffauto_rs::ffmpeg::enums::StatsMode;
use ffauto_rs::ffmpeg::ffprobe::ffprobe;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::Video;

use crate::commands::{Cli, QuantArgs};
use crate::common::{add_basic_filters, add_palette_filters, debug_pause, generate_palette_filtergraph, handle_seek};
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_quant(cli: &Cli, args: &QuantArgs) -> Result<()> {
	let probe = ffprobe(&args.input, false)?;

	let first_video_stream = probe.iter().find(|s| s.codec_type == Video);
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	// add input -t argument to ensure ffmpeg only reads one frame
	ffmpeg_args.push_str("-t");
	if let Some(fps) = video_stream.frame_rate() {
		// if we know the input video's frame rate, we can accurately limit the number of read frames to just one
		ffmpeg_args.push(format!("{}", 1.0/fps));
	} else {
		// else we just say "take the first second's worth of frames" and hope for the best
		ffmpeg_args.push_str("1");
	}

	handle_seek(&mut ffmpeg_args, &args.input, &cli.seek);

	ffmpeg_args.push_str("-an");
	ffmpeg_args.push_str("-dn");
	ffmpeg_args.push_str("-sn");
	ffmpeg_args.push_str("-frames:v");
	ffmpeg_args.push_str("1");
	ffmpeg_args.push_str("-update");
	ffmpeg_args.push_str("1");

	// region Video Filtering

	let mut video_filter: Vec<String> = vec![];
	video_filter.push_str("select=eq(n\\,0)");

	add_basic_filters(&mut video_filter, cli, video_stream.color_transfer.unwrap_or_default())?;

	add_palette_filters(&mut video_filter, args.brightness, args.contrast, args.saturation, args.sharpness);

	let video_filter_str = video_filter.join(",");
	let filter_complex = generate_palette_filtergraph(
		true, false,
		video_filter_str,
		&args.palette_file, &args.palette_name,
		args.num_colors, &StatsMode::default(), false,
		&args.dither, args.bayer_scale,
	)?;

	ffmpeg_args.push_str("-filter_complex");
	ffmpeg_args.push(filter_complex);

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
		return Err(anyhow!("ffmpeg exited with status code {}", exit_status.code().unwrap_or(-1)));
	}

	let execution_time = start.elapsed();
	println!("Encoding took {:.2}s!", execution_time.as_secs_f64());

	Ok(())
}