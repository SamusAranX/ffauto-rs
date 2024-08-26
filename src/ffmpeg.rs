use std::process::Command;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::Result;
use ffauto_rs::ffmpeg_enums::VideoCodec;
use ffauto_rs::ffprobe::ffprobe;
use ffauto_rs::ffprobe_struct::StreamType::*;

use crate::args::ProgramArgs;

pub fn ffmpeg(args: &ProgramArgs) -> Result<()> {
	let start = Instant::now();

	let probe = ffprobe(&args.input).expect("welp");
	println!("ffprobe: {:?}", probe);

	let first_audio_stream = probe.iter().find(|s| s.codec_type == Audio);
	let first_video_stream = probe.iter().find(|s| s.codec_type == Video);

	if first_audio_stream.is_none() && first_video_stream.is_none() {
		return Err(anyhow!("The input file contains no usable audio/video streams"));
	}

	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	// TODO: ffmpeg args builder

	let start_time = args.seek.clone().unwrap_or("0".to_string()).parse::<f64>().unwrap();
	let duration = &video_stream.duration.expect("Can't read video stream duration").parse::<f64>().unwrap();

	let s = format!("{start_time:.2}");
	let mut ffmpeg_args: Vec<&str> = vec![
		"-ss", s.as_str(),
		"-i", args.input.to_str().unwrap()
	];

	// region Audio Filtering

	if first_audio_stream.is_none() || args.mute {
		// input has no audio streams or explicit mute was requested
		ffmpeg_args.push("-an");
	} else if let Some(audio_stream) = first_audio_stream.cloned() {
		if args.audio_copy_possible(audio_stream.codec_name) {
			// input stream is already aac, copy stream
			ffmpeg_args.push("-c:a");
			ffmpeg_args.push("copy");
		} else {
			// input stream is not aac or filtering was requested, do transcode
			ffmpeg_args.push("-c:a");
			ffmpeg_args.push("aac");
			ffmpeg_args.push("-b:a");
			ffmpeg_args.push("256k");

			if args.needs_audio_filter() {
				ffmpeg_args.push("-af");

				let mut audio_filter: Vec<String> = vec![];

				if args.audio_volume != 1.0 {
					audio_filter.push(format!("volume={:.2}", args.audio_volume));
				}

				// 	filter_afadein = f"afade=t=in:st={fadein_start}:d={args.fadein}:curve=losi" if args.fadein else None
				// 	filter_afadeout = f"afade=t=out:st={fadeout_start}:d={args.fadeout}:curve=losi" if args.fadeout else None

				let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
				if args.fade != 0.0 {
					fade_in = args.fade;
					fade_out = args.fade;
				}

				if fade_in > 0.0 {
					audio_filter.push(format!("afade=t=in:st=0:d={:.2}:curve=losi", fade_in));
				}
				if fade_out > 0.0 {
					audio_filter.push(format!("afade=t=out:st=0:d={:.2}:curve=losi", fade_in));
				}
			}
		}
	}

	// endregion

	let mut ffmpeg = Command::new("ffmpeg")
		.arg("-hide_banner")
		.arg("-loglevel").arg("error")
		.args(ffmpeg_args)
		.spawn().expect("failed to run ffmpeg");

	let exit_status = ffmpeg.wait().expect("failed to wait for ffmpeg");
	if !exit_status.success() {
		return Err(anyhow!("ffmpeg exited with status code {}", exit_status.code().unwrap_or(-1)));
	}

	let execution_time = start.elapsed();

	println!("Encoding took {:.2}s!", execution_time.as_secs_f64());

	// let output: FFProbeOutput = serde_json::from_str(stdout.as_str()).expect("failed to deserialize ffprobe output");

	Ok(())
}