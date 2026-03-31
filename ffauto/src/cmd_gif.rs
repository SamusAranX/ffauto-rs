use std::time::Duration;

use crate::commands::GIFArgs;
use crate::common::*;
use crate::palettes::get_builtin_palette;
use crate::vec_push_ext::PushStrExt;
use anyhow::Result;
use ffmpeg::ffmpeg::ffmpeg::ffmpeg;
use ffmpeg::filters::{
	Crop, Fade, FilterChain, FilterChainList, Fps, Palettegen, PalettegenStatsMode, Paletteuse,
	PaletteuseDiffMode, SetSar, Split,
};
use ffmpeg::palettes::palette::Palette;

pub(crate) fn ffmpeg_gif(args: &GIFArgs, debug: bool) -> Result<()> {
	let probe = ffprobe_output(&args.input)?;

	let (video_stream, video_stream_id) =
		probe.checked_get_video_stream_by_index_or_language(&args.video_language, args.video_stream)?;

	let video_duration = probe.duration()?;

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(),
		"error".to_string(),
		"-y".to_string(),
	];

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

	let mut video_pipelines = FilterChainList::new();
	let mut filter_pipeline =
		FilterChain::with_inputs_and_outputs([video_stream_id], ["filtered_paletteuse"]);

	if let Some(fps) = args.framerate {
		filter_pipeline.push(Fps::new(fps));
	}

	if let Some(Ok(crop)) = args.crop.clone().map(Crop::from_arg) {
		filter_pipeline.push(crop);
	}

	if let Some(scale_filter) =
		generate_scale_filter(args.width, args.height, args.size.as_ref(), args.scale_mode)
	{
		filter_pipeline.push(scale_filter);
	}

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

	let mut palettegen_pipeline = FilterChainList::new();
	match (&args.palette_file, &args.palette_name) {
		(Some(palette_file), None) => {
			palettegen_pipeline.extend(match Palette::load_from_file(palette_file) {
				Ok(pal) => palette_to_ffmpeg(&pal),
				Err(e) => anyhow::bail!(e),
			});
		}
		(None, Some(palette_name)) => {
			palettegen_pipeline.extend(palette_to_ffmpeg(&get_builtin_palette(palette_name)));
		}
		(None, None) => {
			// Add a split and an additional "filtered_palettegen" output to the filter pipeline
			// The palettegen filter needs that connected to its input to work
			filter_pipeline.push(Split::new(2));
			filter_pipeline
				.outputs
				.push("filtered_palettegen".to_string());

			let mut palettegen_chain =
				FilterChain::with_inputs_and_outputs(["filtered_palettegen"], ["palette"]);
			palettegen_chain.push(Palettegen::new(args.num_colors, false, args.stats_mode));

			palettegen_pipeline.push(palettegen_chain);
		}
		_ => anyhow::bail!("Well, this wasn't supposed to happen."),
	}

	video_pipelines.push(filter_pipeline);
	video_pipelines.extend(palettegen_pipeline);

	let diff_mode = if args.diff_rect {
		PaletteuseDiffMode::Rectangle
	} else {
		PaletteuseDiffMode::None
	};
	let new_palette = args.palette_file.is_none()
		&& args.palette_name.is_none()
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
