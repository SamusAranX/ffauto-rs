use std::time::Duration;

use crate::commands::GIFArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;
use anyhow::Result;
use clap::ArgMatches;
use ffmpeg::ffmpeg::ffmpeg::ffmpeg;
use ffmpeg::ffmpeg::ffmpeg_cropdetect::ffmpeg_cropdetect;
use ffmpeg::filters::{
	Crop, Fade, FilterChain, FilterChainList, Fps, PalettegenStatsMode, Paletteuse, PaletteuseDiffMode,
	SetSar,
};

pub(crate) fn ffmpeg_gif(args: &GIFArgs, matches: &ArgMatches, debug: bool) -> Result<()> {
	let mut remove_bars_crop: Option<Crop> = None;
	if args.remove_bars {
		eprintln!("Gathering autocrop information…");
		remove_bars_crop = Some(ffmpeg_cropdetect(&args.input)?);
	}

	let probe = ffprobe_output(&args.input)?;

	let (video_stream, video_stream_id) =
		probe.checked_get_video_stream_by_index_or_language(&args.video_language, args.video_stream)?;

	let video_duration = probe.duration()?;

	let mut ffmpeg_args: Vec<String> = Vec::new();

	let seek = args.parse_seek();
	let duration = args.parse_duration();

	if let Some(seek) = seek {
		ffmpeg_args.add_two("-ss", format!("{}", seek.as_secs_f64()));
	}

	// input option to limit the amount of data read
	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);

	// repeat as output option to limit the amount of data written
	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	ffmpeg_args.add("-an");
	ffmpeg_args.add("-dn");
	ffmpeg_args.add("-sn");

	// region Video Filtering

	// get order of crop and scale arguments so we can reorder the crop and scale filters below
	let (crop_index, scale_index) = get_crop_and_scale_order(matches);

	let mut video_pipelines = FilterChainList::new();
	let mut filter_pipeline =
		FilterChain::with_inputs_and_outputs([video_stream_id], ["filtered_paletteuse"]);

	if let Some(remove_bars_crop) = remove_bars_crop {
		filter_pipeline.push(remove_bars_crop);
	}

	if let Some(fps) = args.framerate {
		filter_pipeline.push(Fps::new(fps));
	}

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

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade > 0.0 {
		fade_in = args.fade;
		fade_out = args.fade;
	}
	let fade_out_start = if let Some(duration) = duration {
		// duration was given
		duration.as_secs_f64() - fade_out
	} else {
		// duration wasn't given, use video duration
		video_duration
			.saturating_sub(seek.unwrap_or(Duration::ZERO))
			.as_secs_f64()
			- fade_out
	};

	if fade_in > 0.0 {
		filter_pipeline.push(Fade::r#in(0.0, fade_in));
	}
	if fade_out > 0.0 {
		filter_pipeline.push(Fade::out(fade_out_start, fade_out));
	}

	filter_pipeline.push(SetSar::square());

	let palettegen_pipeline = generate_palette_filter(
		args.palette_file.clone(),
		args.palette_static.clone(),
		args.palette_dynamic.clone(),
		args.palette_gradient.clone(),
		args.palette_steps,
		&mut filter_pipeline,
		args.stats_mode,
	)?;

	video_pipelines.push(filter_pipeline);
	video_pipelines.extend(palettegen_pipeline);

	let diff_mode = if args.diff_rect {
		PaletteuseDiffMode::Rectangle
	} else {
		PaletteuseDiffMode::None
	};
	let new_palette = args.palette_file.is_none()
		&& args.palette_static.is_none()
		&& args.palette_dynamic.is_none()
		&& args.palette_gradient.is_none()
		&& args.stats_mode == PalettegenStatsMode::Single;

	let mut paletteuse_chain = FilterChain::with_inputs(["filtered_paletteuse", "palette"]);
	paletteuse_chain.push(Paletteuse::new(
		args.dither,
		args.bayer_scale,
		diff_mode,
		new_palette,
	));

	video_pipelines.push(paletteuse_chain);

	let filter_string = video_pipelines.to_string();
	ffmpeg_args.add_two("-filter_complex", filter_string);

	// endregion

	if args.dedup {
		ffmpeg_args.add_two("-fps_mode", "vfr");
	}
	ffmpeg_args.add_two("-f", "gif");
	ffmpeg_args.add_two("-loop", "0");

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	ffmpeg(&ffmpeg_args, None, false, debug)
}
