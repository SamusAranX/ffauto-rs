use crate::cmd::{handle_duration, handle_seek, palette_to_ffmpeg};
use crate::commands::{Cli, GIFArgs};
use crate::vec_push_ext::PushStrExt;
use anyhow::anyhow;
use anyhow::Result;
use ffauto_rs::ffmpeg_enums::{Crop, StatsMode};
use ffauto_rs::ffprobe::ffprobe;
use ffauto_rs::ffprobe_struct::StreamType::Video;
use ffauto_rs::palettes::palette::Palette;
use std::io;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

pub(crate) fn ffmpeg_gif(cli: &Cli, args: &GIFArgs) -> Result<()> {
	let start = Instant::now();

	let probe = ffprobe(&args.input, false).expect("welp");

	let first_video_stream = probe.iter().find(|s| s.codec_type == Video);
	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();
	let video_stream_duration = video_stream.duration.clone().expect("Can't read video stream duration").parse::<f64>().unwrap();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "error".to_string(),
		"-y".to_string(),
	];

	handle_seek(&mut ffmpeg_args, &args.input, &cli.seek);
	handle_duration(&mut ffmpeg_args, &cli.seek, &args.duration, &args.duration_to);

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade != 0.0 {
		fade_in = args.fade;
		fade_out = args.fade;
	}
	let fade_out_start = video_stream_duration - fade_out;

	// region Video Filtering

	let mut video_filter: Vec<String> = vec![];

	if let Some(fps) = args.framerate {
		video_filter.push(format!("fps=fps={:.3}", fps));
	}

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

	video_filter.push(format!("eq=brightness={}:saturation={}:contrast={}", args.brightness, args.saturation, args.contrast));
	video_filter.push(format!("unsharp=la={0}:ca={0}", args.sharpness));

	if fade_in > 0.0 {
		video_filter.push(format!("fade=t=in:st=0:d={fade_in:.3}"));
	}
	if fade_out > 0.0 {
		video_filter.push(format!("fade=t=out:st={fade_out_start:.3}:d={fade_out:.3}"));
	}

	let video_filter_str = video_filter.join(",");

	ffmpeg_args.push_str("-filter_complex");

	let diff_mode = if args.diff_rect { "rectangle" } else { "none" };
	let new_palette = if args.stats_mode == StatsMode::Single { 1 } else { 0 };

	let filter_complex = {
		if let Some(palette_file) = &args.palette_file {
			let pal = Palette::load_from_file(palette_file).map_err(|e| anyhow!(e))?;
			let pal_string = palette_to_ffmpeg(pal);

			[
				format!("{pal_string} [pal]"),
				format!("[0:v] {video_filter_str} [filtered];[filtered][pal] paletteuse=dither={}:bayer_scale={}:diff_mode={diff_mode}:new={new_palette}", args.dither, args.bayer_scale),
			].join(";")
		} else {
			[
				format!("[0:v] {video_filter_str},split [a][b]"),
				format!("[a] palettegen=max_colors={}:reserve_transparent=0:stats_mode={} [pal]", args.num_colors, args.stats_mode),
				format!("[b][pal] paletteuse=dither={}:bayer_scale={}:diff_mode={diff_mode}:new={new_palette}", args.dither, args.bayer_scale),
			].join(";")
		}
	};

	ffmpeg_args.push(filter_complex);

	// endregion

	ffmpeg_args.push_str("-f");
	ffmpeg_args.push_str("gif");
	ffmpeg_args.push_str("-loop");
	ffmpeg_args.push_str("0");

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