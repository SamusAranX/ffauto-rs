use std::io;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

use crate::args::ProgramArgs;
use crate::vec_push_ext::PushStrExt;
use anyhow::anyhow;
use anyhow::Result;
use ffauto_rs::ffmpeg_enums::{Crop, VideoCodec};
use ffauto_rs::ffprobe::ffprobe;
use ffauto_rs::ffprobe_struct::StreamType::*;
use ffauto_rs::timestamps::parse_ffmpeg_timestamp;

pub fn ffmpeg(args: &ProgramArgs) -> Result<()> {
	let start = Instant::now();

	let probe = ffprobe(&args.input, false).expect("welp");
	// println!("ffprobe: {:?}", probe);

	let first_audio_stream = probe.iter().find(|s| s.codec_type == Audio);
	let first_video_stream = probe.iter().find(|s| s.codec_type == Video);

	if first_audio_stream.is_none() && first_video_stream.is_none() {
		return Err(anyhow!("The input file contains no usable audio/video streams"));
	}

	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();
	let video_stream_duration = video_stream.duration.clone().expect("Can't read video stream duration").parse::<f64>().unwrap();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "warning".to_string(),
		"-y".to_string(),
	];

	if let Some(ss) = &args.seek {
		ffmpeg_args.push_str("-ss");
		ffmpeg_args.push(format!("{}", parse_ffmpeg_timestamp(ss).unwrap_or_default().as_secs_f64()));
	}

	ffmpeg_args.push_str("-i");
	ffmpeg_args.push(args.input.to_str().unwrap().to_string());

	if let Some(t) = &args.duration {
		match parse_ffmpeg_timestamp(t) {
			Some(t) => {
				ffmpeg_args.push_str("-t");
				ffmpeg_args.push(format!("{}", t.as_secs_f64()));
			}
			None => { eprintln!("invalid duration string: {t}") }
		}
	} else if let Some(to) = &args.duration_to {
		match parse_ffmpeg_timestamp(to) {
			Some(to) => {
				ffmpeg_args.push_str("-to");
				ffmpeg_args.push(format!("{}", to.as_secs_f64()));
			}
			None => { eprintln!("invalid duration string: {to}") }
		}
	}

	ffmpeg_args.push_str("-preset");
	ffmpeg_args.push(args.preset.to_string());

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade != 0.0 {
		fade_in = args.fade;
		fade_out = args.fade;
	}
	let fade_out_start = video_stream_duration - fade_out;

	// region Audio Filtering

	if first_audio_stream.is_none() || args.mute {
		// input has no audio streams or explicit mute was requested
		ffmpeg_args.push_str("-an");
	} else if let Some(audio_stream) = first_audio_stream.cloned() {
		if args.audio_copy_possible(audio_stream.codec_name) {
			// input stream is already aac, copy stream
			ffmpeg_args.push_str("-c:a");
			ffmpeg_args.push_str("copy");
		} else {
			// input stream is not aac or filtering was requested, do transcode
			ffmpeg_args.push_str("-c:a");
			ffmpeg_args.push_str(args.video_codec.audio_codec());
			ffmpeg_args.push_str("-b:a");
			ffmpeg_args.push_str("256k");

			if args.needs_audio_filter() {
				let mut audio_filter: Vec<String> = vec![];

				if args.audio_volume != 1.0 {
					audio_filter.push(format!("volume={:.3}", args.audio_volume));
				}

				if fade_in > 0.0 {
					audio_filter.push(format!("afade=t=in:st=0:d={fade_in:.3}:curve=losi"));
				}
				if fade_out > 0.0 {
					audio_filter.push(format!("afade=t=out:st={fade_out_start:.3}:d={fade_out:.3}:curve=losi"));
				}

				let audio_filter_str = audio_filter.join(",");
				ffmpeg_args.push_str("-af");
				ffmpeg_args.push(audio_filter_str);
			}
		}
	}

	// endregion

	// region Video Filtering

	let crf = format!("{}", &args.video_codec.crf_with_garbage(args.garbage));
	ffmpeg_args.push_str("-c:v");
	ffmpeg_args.push_str(args.video_codec.video_codec());
	ffmpeg_args.push_str("-crf");
	ffmpeg_args.push_str(&crf);
	ffmpeg_args.push_str("-pix_fmt");
	ffmpeg_args.push_str(args.video_codec.pix_fmt());
	ffmpeg_args.push_str("-tune");
	match args.video_codec {
		VideoCodec::H264 => { ffmpeg_args.push_str("film"); }
		VideoCodec::H265 | VideoCodec::H265_10 => {
			ffmpeg_args.push_str("grain");
			ffmpeg_args.push_str("-tag:v");
			ffmpeg_args.push_str("hvc1");
		}
	}

	if args.faststart {
		ffmpeg_args.push_str("-movflags");
		ffmpeg_args.push_str("faststart");
	}

	if args.needs_video_filter() {
		let mut video_filter: Vec<String> = vec![];

		if let Some(fps) = args.framerate {
			// let video_fps = video_stream.frame_rate().unwrap();
			// let frames = (video_fps / fps).round() as i64;
			// video_filter.push(format!("tmix=frames={frames}"));
			video_filter.push(format!("fps=fps={:.3}", fps));
		}

		if let Some(crop) = Crop::new(&args.crop.clone().unwrap_or_default()) {
			video_filter.push(format!("crop={crop}"));
		}

		if let Some(width) = args.width {
			video_filter.push(format!("scale=w={width}:h=-2:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", args.scale_mode));
		} else if let Some(height) = args.height {
			video_filter.push(format!("scale=w=-2:h={height}:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", args.scale_mode));
		}

		let color_transfer = video_stream.color_transfer.unwrap_or_default();
		if args.tonemap && (color_transfer.contains("smpte2084") || color_transfer.contains("arib-std-b67")) {
			video_filter.push_str("zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709");
		} else if args.tonemap {
			eprintln!("HDR-to-SDR tonemap requested but input video is already SDR");
		}

		if fade_in > 0.0 {
			video_filter.push(format!("fade=t=in:st=0:d={fade_in:.3}"));
		}
		if fade_out > 0.0 {
			video_filter.push(format!("fade=t=out:st={fade_out_start:.3}:d={fade_out:.3}"));
		}

		let video_filter_str = video_filter.join(",");
		ffmpeg_args.push_str("-vf");
		ffmpeg_args.push(video_filter_str);
	}

	// endregion

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	if args.debug {
		println!("{:#^40}", " DEBUG MODE ");
		println!("ffmpeg args: {:?}", args);
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