use anyhow::Result;

use crate::commands::QuantArgs;
use crate::common::*;
use crate::palettes::get_builtin_palette;
use crate::vec_push_ext::PushStrExt;
use ffmpeg::ffmpeg::ffmpeg::ffmpeg;
use ffmpeg::ffmpeg::ffprobe::ffprobe;
use ffmpeg::filters::{
	Crop, FilterChain, FilterChainList, Palettegen, PalettegenStatsMode, Paletteuse, PaletteuseDiffMode,
	Select, SetSar, Split,
};
use ffmpeg::palettes::palette::Palette;

pub(crate) fn ffmpeg_quant(args: &QuantArgs, debug: bool) -> Result<()> {
	let probe = ffprobe(&args.input, false)?;

	let (video_stream, video_stream_id) =
		probe.checked_get_video_stream_by_index_or_language(&args.video_language, args.video_stream)?;

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(),
		"error".to_string(),
		"-y".to_string(),
	];

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

	let mut video_pipelines = FilterChainList::new();
	let mut filter_pipeline = FilterChain::with_inputs_and_outputs(
		vec![video_stream_id],
		vec!["filtered1".to_string(), "filtered2".to_string()],
	);

	filter_pipeline.push(Select::new("eq(n\\,0)", 1));

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

	filter_pipeline.push(SetSar::square());
	filter_pipeline.push(Split::new(2));

	video_pipelines.push(filter_pipeline);

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
			let mut palettegen_chain = FilterChain::with_inputs_and_outputs(
				vec!["filtered1".to_string()],
				vec!["palette".to_string()],
			);
			palettegen_chain.push(Palettegen::new(args.num_colors, false, PalettegenStatsMode::Full));

			palettegen_pipeline.push(palettegen_chain);
		}
		_ => anyhow::bail!("Well, this wasn't supposed to happen."),
	}

	video_pipelines.extend(palettegen_pipeline);

	let new_palette = args.palette_file.is_none() && args.palette_name.is_none();

	let mut paletteuse_chain = FilterChain::with_inputs(vec!["filtered2".to_string(), "palette".to_string()]);
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
