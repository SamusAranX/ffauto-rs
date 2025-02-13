use anyhow::Result;

use crate::commands::BarcodeArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;
use ffauto_rs::ffmpeg::enums::BarcodeMode;
use ffauto_rs::ffmpeg::ffmpeg::ffmpeg;

pub(crate) fn ffmpeg_barcode(args: &BarcodeArgs, debug: bool) -> Result<()> {
	let probe = ffprobe_frames(&args.input)?;

	let first_video_stream = probe.get_first_video_stream();
	if first_video_stream.is_none() {
		anyhow::bail!("The input file contains no usable audio/video streams")
	}

	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();

	match video_stream.height {
		None => anyhow::bail!("The selected video stream contains no height information"),
		Some(0) => anyhow::bail!("The selected video stream contains invalid height information"),
		_ => (),
	}

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "warning".to_string(),
		"-y".to_string(),
	];

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);

	// region Filtering

	let mut video_pipelines: Vec<Vec<String>> = vec!();
	let mut input_pipeline: Vec<String> = vec!();

	if video_stream.is_hdr() {
		input_pipeline.push(format!("[0:v]{TONEMAP_FILTER}"));
	}

	let scale_flags = "accurate_rnd+full_chroma_int+full_chroma_inp";
	let video_height = video_stream.height.unwrap();
	let video_frames = video_stream.total_frames().unwrap();

	input_pipeline.add("format=rgb48be");

	match args.barcode_mode {
		BarcodeMode::Frames => {
			input_pipeline.add(format!("scale=w=1:h={video_height}:flags=bicubic+{scale_flags}"));
			input_pipeline.add(format!("tile={video_frames}x1 [out]"));
			video_pipelines.push(input_pipeline);
		}
		BarcodeMode::Colors => {
			input_pipeline.add(format!("scale=w=320:h=-2:flags=bicubic+{scale_flags}"));
			input_pipeline.add("colorspace=all=bt709:trc=srgb:range=pc"); // palettegen complains if this isn't here
			input_pipeline.add("palettegen=max_colors=2:reserve_transparent=0:stats_mode=single");
			input_pipeline.add("split [p1][p2]");
			video_pipelines.push(input_pipeline);

			video_pipelines.push(vec!(
				"[p1] crop=1:1:0:0".to_string(), // dark
				format!("scale=w=1:h={video_height}:flags=neighbor+{scale_flags}"),
				format!("tile={video_frames}x1 [s1]"),
			));
			video_pipelines.push(vec!(
				"[p2] crop=1:1:1:0".to_string(), // light
				format!("scale=w=1:h={video_height}:flags=neighbor+{scale_flags}"),
				format!("tile={video_frames}x1 [s2]"),
			));
			video_pipelines.push(vec!(
				"[s2][s1] blend=all_mode=softlight [out]".to_string()
			));
		}
	}

	let mut output_pipeline = vec!();
	output_pipeline.add("[out] setsar=1");
	if args.deep_color {
		output_pipeline.add("format=rgb48be");
	} else {
		output_pipeline.add("format=rgb24");
	}
	if let Some(output_height) = args.height {
		output_pipeline.add(format!("scale=h={output_height}:flags=bicubic+{scale_flags}"));
	}
	output_pipeline.add("setparams=colorspace=bt709:color_primaries=bt709:color_trc=iec61966-2-1");
	video_pipelines.push(output_pipeline);

	let filter_graph = video_pipelines.iter().map(|p| p.join(",")).collect::<Vec<String>>().join(";");
	ffmpeg_args.add_two("-filter_complex", filter_graph);

	// endregion

	ffmpeg_args.add_two("-c:v", "png");
	ffmpeg_args.add_two("-f", "image2");
	ffmpeg_args.add_two("-update", "1");

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	ffmpeg(&ffmpeg_args, false, debug)
}
