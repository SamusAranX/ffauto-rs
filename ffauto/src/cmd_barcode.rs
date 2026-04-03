use crate::commands::BarcodeArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;
use anyhow::Result;
use ffmpeg::ffmpeg::enums::BarcodeMode;
use ffmpeg::ffmpeg::ffmpeg::ffmpeg;
use ffmpeg::ffmpeg::ffprobe::ffprobe;
use ffmpeg::filters::{
	Blend, BlendMode, Colorspace, Crop, FilterChain, FilterChainList, Format, Palettegen,
	PalettegenStatsMode, Scale, ScaleAlgorithm, SetParams, SetSar, Split, Tile,
};

pub(crate) fn ffmpeg_barcode(args: &BarcodeArgs, debug: bool) -> Result<()> {
	eprintln!("Gathering media information…");

	// ffprobe_frames() bails if the input file has no usable video streams
	let probe = match args.video_frames {
		None => ffprobe_frames(&args.input)?,
		Some(_) => ffprobe(&args.input, false)?,
	};

	let mut ffmpeg_args: Vec<String> = vec!["-hide_banner", "-loglevel", "warning", "-y"]
		.into_iter()
		.map(Into::into)
		.collect();

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);

	ffmpeg_args.add("-an");
	ffmpeg_args.add("-dn");
	ffmpeg_args.add("-sn");

	let (video_stream, video_stream_id) =
		probe.checked_get_video_stream_by_index_or_language(&args.video_language, args.video_stream)?;

	let video_height = video_stream.height.unwrap();
	let video_frames = &args
		.video_frames
		.unwrap_or_else(|| video_stream.total_frames().unwrap());
	check_frame_size(*video_frames, video_height)?;

	#[cfg(debug_assertions)]
	eprintln!("total frames: {video_frames}");

	// region Filtering

	let mut video_pipelines = FilterChainList::new();
	let mut input_pipeline = FilterChain::with_inputs_and_outputs([video_stream_id], ["video_out"]);

	if video_stream.is_hdr() {
		input_pipeline.extend(sdr_tonemap_chain());
	}

	// unconditionally convert to rgb48 here so there's more color to work with before the final output
	input_pipeline.push(Format::new("rgb48be"));

	#[allow(clippy::cast_possible_truncation)]
	match args.barcode_mode {
		BarcodeMode::Frames => {
			input_pipeline.push(Scale::column(video_height as i32, ScaleAlgorithm::Bicubic));
			input_pipeline.push(Tile::columns(*video_frames));
			video_pipelines.push(input_pipeline);
		}
		BarcodeMode::Colors => {
			input_pipeline.set_outputs(["p1", "p2"]);
			input_pipeline.push(Scale::preserve_aspect_ratio_width(320, ScaleAlgorithm::default()));
			input_pipeline.push(Colorspace::srgb()); // palettegen complains if this isn't here
			input_pipeline.push(Palettegen::new(2, false, PalettegenStatsMode::Single));
			input_pipeline.push(Split::new(2));
			video_pipelines.push(input_pipeline);

			let mut light_pixel_pipeline = FilterChain::with_inputs_and_outputs(["p1"], ["s1"]);
			light_pixel_pipeline.push(Crop::new(1, 1, 0, 0));
			light_pixel_pipeline.push(Scale::column(video_height as i32, ScaleAlgorithm::Neighbor));
			light_pixel_pipeline.push(Tile::columns(*video_frames));
			video_pipelines.push(light_pixel_pipeline);

			let mut dark_pixel_pipeline = FilterChain::with_inputs_and_outputs(["p2"], ["s2"]);
			dark_pixel_pipeline.push(Crop::new(1, 1, 1, 0));
			dark_pixel_pipeline.push(Scale::column(video_height as i32, ScaleAlgorithm::Neighbor));
			dark_pixel_pipeline.push(Tile::columns(*video_frames));
			video_pipelines.push(dark_pixel_pipeline);

			let mut blend_pipeline = FilterChain::with_inputs_and_outputs(["s1", "s2"], ["video_out"]);
			blend_pipeline.push(Blend::new(BlendMode::SoftLight));
			video_pipelines.push(blend_pipeline);
		}
	}

	let mut output_pipeline = FilterChain::with_inputs(["video_out"]);
	output_pipeline.push(SetSar::square());

	if args.deep_color {
		output_pipeline.push(Format::new("rgb48be"));
	} else {
		output_pipeline.push(Format::new("rgb24"));
	}

	#[allow(clippy::cast_possible_truncation)]
	if let Some(output_height) = args.height {
		output_pipeline.push(Scale::new(
			*video_frames as i32,
			output_height as i32,
			ScaleAlgorithm::Neighbor,
		));
	}

	output_pipeline.push(SetParams::srgb());
	video_pipelines.push(output_pipeline);

	let filter_string = video_pipelines.to_string();
	ffmpeg_args.add_two("-filter_complex", filter_string);

	// endregion

	ffmpeg_args.add_two("-c:v", "png");
	ffmpeg_args.add_two("-f", "image2");
	ffmpeg_args.add_two("-update", "1");

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	eprintln!("Generating image…");

	ffmpeg(&ffmpeg_args, None, false, debug)
}
