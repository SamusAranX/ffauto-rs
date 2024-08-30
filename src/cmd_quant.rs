use crate::commands::{Cli, QuantArgs};
use crate::common::{debug_pause, generate_full_filtergraph, generate_scale_filter, handle_seek};
use crate::vec_push_ext::PushStrExt;
use anyhow::anyhow;
use anyhow::Result;
use ffauto_rs::ffmpeg::enums::{Crop, StatsMode};
use ffauto_rs::ffmpeg::ffprobe::ffprobe;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::Video;
use std::process::Command;
use std::time::Instant;

pub(crate) fn ffmpeg_quant(cli: &Cli, args: &QuantArgs) -> Result<()> {
	let probe = ffprobe(&args.input, false).expect("welp");

	let first_video_stream = probe.iter().find(|s| s.codec_type == Video);
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	handle_seek(&mut ffmpeg_args, &args.input, &cli.seek);

	ffmpeg_args.push_str("-frames");
	ffmpeg_args.push_str("1");

	// region Video Filtering

	let mut video_filter: Vec<String> = vec![];

	if let Some(crop) = Crop::new(&cli.crop.clone().unwrap_or_default()) {
		video_filter.push(format!("crop={crop}"));
	}

	if let Some(scale) = generate_scale_filter(cli) {
		video_filter.push(scale);
	}

	let color_transfer = video_stream.color_transfer.unwrap_or_default();
	if color_transfer.contains("smpte2084") || color_transfer.contains("arib-std-b67") {
		video_filter.push_str("zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709");
	}

	video_filter.push(format!("eq=brightness={}:saturation={}:contrast={}", args.brightness, args.saturation, args.contrast));
	video_filter.push(format!("unsharp=la={0}:ca={0}", args.sharpness));

	let video_filter_str = video_filter.join(",");
	let filter_complex = generate_full_filtergraph(
		true, video_filter_str,
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