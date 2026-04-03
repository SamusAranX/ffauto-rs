use crate::commands::QuantArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;
use anyhow::Result;
use clap::ArgMatches;
use ffmpeg::ffmpeg::ffmpeg::ffmpeg;
use ffmpeg::ffmpeg::ffmpeg_cropdetect::ffmpeg_cropdetect;
use ffmpeg::ffmpeg::ffprobe::ffprobe;
use ffmpeg::filters::{
	Crop, FilterChain, FilterChainList, PalettegenStatsMode, Paletteuse, PaletteuseDiffMode, Select, SetSar,
};

pub(crate) fn ffmpeg_quant(args: &QuantArgs, matches: &ArgMatches, debug: bool) -> Result<()> {
	let mut remove_bars_crop: Option<Crop> = None;
	if args.remove_bars {
		eprintln!("Gathering autocrop information…");
		remove_bars_crop = Some(ffmpeg_cropdetect(&args.input)?);
	}

	let probe = ffprobe(&args.input, false)?;

	let (video_stream, video_stream_id) =
		probe.checked_get_video_stream_by_index_or_language(&args.video_language, args.video_stream)?;

	let mut ffmpeg_args: Vec<String> = Vec::new();

	let seek = args.parse_seek();
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

	// get order of crop and scale arguments so we can reorder the crop and scale filters below
	let (crop_index, scale_index) = get_crop_and_scale_order(matches);

	let mut video_pipelines = FilterChainList::new();
	let mut filter_pipeline =
		FilterChain::with_inputs_and_outputs([video_stream_id], ["filtered_paletteuse"]);

	if let Some(remove_bars_crop) = remove_bars_crop {
		filter_pipeline.push(remove_bars_crop);
	}

	filter_pipeline.push(Select::new("eq(n\\,0)", 1));

	let mut crop_and_scale = FilterChain::new();

	if let Some(Ok(crop)) = args.crop.clone().map(Crop::from_arg) {
		crop_and_scale.push(crop);
	}

	if let Some(scale_filter) = generate_scale_filter(
		args.width,
		args.height,
		args.size_fit.as_ref(),
		args.size_fill.as_ref(),
		args.scale_mode,
	) {
		crop_and_scale.push(scale_filter);
	}

	if scale_index < crop_index {
		// if the scale argument was provided before the crop argument,
		// flip this list around
		crop_and_scale.reverse();
	}
	filter_pipeline.extend(crop_and_scale);

	if video_stream.is_hdr() {
		filter_pipeline.extend(sdr_tonemap_chain());
	}

	if let Some(color_filters) = args.generate_color_filters() {
		filter_pipeline.extend(color_filters);
	}

	filter_pipeline.push(SetSar::square());

	let palettegen_pipeline = generate_palette_filter(
		args.palette_file.clone(),
		args.palette_static.clone(),
		args.palette_dynamic.clone(),
		args.palette_gradient.clone(),
		args.palette_steps,
		&mut filter_pipeline,
		PalettegenStatsMode::Full,
	)?;

	video_pipelines.push(filter_pipeline);
	video_pipelines.extend(palettegen_pipeline);

	let new_palette = args.palette_file.is_none()
		&& args.palette_static.is_none()
		&& args.palette_dynamic.is_none()
		&& args.palette_gradient.is_none();

	let mut paletteuse_chain = FilterChain::with_inputs(["filtered_paletteuse", "palette"]);
	paletteuse_chain.push(Paletteuse::new(
		args.dither,
		args.bayer_scale,
		PaletteuseDiffMode::None,
		new_palette,
	));

	video_pipelines.push(paletteuse_chain);

	let filter_string = video_pipelines.to_string();
	ffmpeg_args.add_two("-filter_complex", filter_string);

	// endregion

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	ffmpeg(&ffmpeg_args, None, false, debug)
}
