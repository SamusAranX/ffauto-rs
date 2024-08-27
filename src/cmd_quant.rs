use crate::cmd::handle_seek;
use crate::commands::{Cli, QuantArgs};
use crate::vec_push_ext::PushStrExt;
use anyhow::anyhow;
use anyhow::Result;
use ffauto_rs::ffmpeg_enums::Crop;
use ffauto_rs::ffprobe::ffprobe;
use ffauto_rs::ffprobe_struct::StreamType::Video;
use std::io;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

pub fn ffmpeg_quant(cli: &Cli, args: &QuantArgs) -> Result<()> {
	let start = Instant::now();

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

	if let Some(width) = cli.width {
		video_filter.push(format!("scale=w={width}:h=-2:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", cli.scale_mode));
	} else if let Some(height) = cli.height {
		video_filter.push(format!("scale=w=-2:h={height}:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", cli.scale_mode));
	}

	let color_transfer = video_stream.color_transfer.unwrap_or_default();
	if color_transfer.contains("smpte2084") || color_transfer.contains("arib-std-b67") {
		video_filter.push_str("zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709");
	}

	// TODO: why are these not working properly
	// convert frames to srgb
	// video_filter.push_str("colorspace=all=bt709:trc=iec61966-2-1:primaries=bt709:range=pc:format=yuv444p");
	// if video_stream.color_range == Some("tv".to_string()) {
	// 	video_filter.push_str("scale=in_range=tv:out_range=pc");
	// }

	video_filter.push(format!("eq=brightness={}:saturation={}:contrast={}", args.brightness, args.saturation, args.contrast));
	video_filter.push(format!("unsharp=la={0}:ca={0}", args.sharpness));

	let video_filter_str = video_filter.join(",");

	ffmpeg_args.push_str("-filter_complex");

	let filter_complex = [
		format!("[0:v] {video_filter_str},split [a][b]"),
		format!("[a] palettegen=max_colors={}:reserve_transparent=0 [pal]", args.num_colors),
		format!("[b][pal] paletteuse=dither={}:bayer_scale={}", args.dither, args.bayer_scale),
	].join(";");

	ffmpeg_args.push(filter_complex);

	// endregion

	// TODO: see TODO above
	// ffmpeg_args.push_str("-color_primaries");
	// ffmpeg_args.push_str("bt709");
	// ffmpeg_args.push_str("-color_trc");
	// ffmpeg_args.push_str("iec61966_2_1");
	// ffmpeg_args.push_str("-colorspace");
	// ffmpeg_args.push_str("rgb");
	// ffmpeg_args.push_str("-color_range");
	// ffmpeg_args.push_str("pc");

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	if cli.debug {
		println!("{:#^40}", " DEBUG MODE ");
		println!("program args: {:?}", args);
		println!("ffmpeg args: {}", ffmpeg_args.join(" "));
		let mut stdout = io::stdout();
		let stdin = io::stdin();
		write!(stdout, "{:#^40}", " Press Enter to continue… ").unwrap();
		stdout.flush().unwrap();
		let _ = stdin.read_line(&mut "".to_string()).unwrap();
		writeln!(stdout, "Continuing…").unwrap();
	}

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