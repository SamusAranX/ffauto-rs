use std::process::Command;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::Result;

use ffauto_rs::ffmpeg::enums::{Crop, VideoCodec};
use ffauto_rs::ffmpeg::ffprobe::ffprobe;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::{Audio, Video};

use crate::commands::{AutoArgs, Cli};
use crate::common::{debug_pause, generate_scale_filter, handle_duration, handle_seek};
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_auto(cli: &Cli, args: &AutoArgs) -> Result<()> {
	let probe = ffprobe(&args.input, false)?;

	let first_audio_stream = probe.iter().find(|s| s.codec_type == Audio);
	let first_video_stream = probe.iter().find(|s| s.codec_type == Video);

	if first_audio_stream.is_none() && first_video_stream.is_none() {
		return Err(anyhow!("The input file contains no usable audio/video streams"));
	}

	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "warning".to_string(),
		"-y".to_string(),
	];

	let seek = handle_seek(&mut ffmpeg_args, &args.input, &cli.seek);
	let duration = handle_duration(&mut ffmpeg_args, seek, &args.duration, &args.duration_to);

	ffmpeg_args.push_str("-preset");
	ffmpeg_args.push(args.preset.to_string());

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade != 0.0 {
		fade_in = args.fade;
		fade_out = args.fade;
	}
	let fade_out_start = duration - fade_out;

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

	if args.needs_video_filter(cli) {
		let mut video_filter: Vec<String> = vec![];

		if let Some(fps) = args.framerate {
			// let video_fps = video_stream.frame_rate().unwrap();
			// let frames = (video_fps / fps).round() as i64;
			// video_filter.push(format!("tmix=frames={frames}"));
			video_filter.push(format!("fps=fps={:.3}", fps));
		}

		if let Some(crop) = Crop::new(&cli.crop.clone().unwrap_or_default()) {
			video_filter.push(format!("crop={crop}"));
		}

		let scale = generate_scale_filter(cli)?;
		if !scale.is_empty() {
			video_filter.push(scale);
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