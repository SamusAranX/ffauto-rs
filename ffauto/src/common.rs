use anyhow::Result;
use ffmpeg::ffmpeg::ffprobe::ffprobe;
use ffmpeg::ffmpeg::ffprobe_struct::{FFProbeOutput, StreamType};
use ffmpeg::ffmpeg::timestamps::parse_ffmpeg_duration;
use ffmpeg::filters::{
	Eq, FilterChain, FilterChainList, Format, Scale, ScaleAlgorithm, ScaleForceOriginalAspectRatio, Split,
	Tonemap, TonemapAlgorithm, Unsharp, Xstack, Zscale, ZscaleMatrix, ZscalePrimaries, ZscaleTransfer,
};
use ffmpeg::palettes::palette::Palette;
use std::path::Path;

use clap::ArgMatches;
use std::time::Duration;

const MAX32: u64 = i32::MAX as u64;

// zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709"

/// Returns the standard tonemap filter chain for HDR-to-SDR conversion.
pub(crate) fn sdr_tonemap_chain() -> FilterChain {
	let mut list = FilterChain::new();

	list.push(Zscale::new_transfer_and_npl(ZscaleTransfer::Linear, 100.0));
	list.push(Format::new("gbrpf32le"));
	list.push(Zscale::new_primaries(ZscalePrimaries::Bt709));
	list.push(Tonemap::new(TonemapAlgorithm::Hable, 0.0));
	list.push(Zscale::new_transfer_and_matrix(
		ZscaleTransfer::Linear,
		ZscaleMatrix::Bt709,
	));

	list
}

pub(crate) fn get_crop_and_scale_order(matches: &ArgMatches) -> (usize, usize) {
	let crop_index = matches.index_of("crop").unwrap_or_default();
	let scale_index = matches
		.index_of("width")
		.or_else(|| matches.index_of("height"))
		.or_else(|| matches.index_of("size"))
		.unwrap_or_default();

	(crop_index, scale_index)
}

pub trait CanSeek {
	fn parse_seek(&self) -> Option<Duration>;
}

pub trait CanSetDuration {
	fn parse_duration(&self) -> Option<Duration>;
}

pub trait CanColorFilter {
	fn generate_color_filters(&self) -> Option<FilterChain>;
}

/// Parses the seek string and returns it as a [Duration], if present.
pub(crate) fn parse_seek(seek: Option<&str>) -> Option<Duration> {
	if let Some(seek_str) = seek {
		return parse_ffmpeg_duration(seek_str);
	}

	None
}

/// Parses the duration strings and returns an appropriate [Duration].
pub(crate) fn parse_duration(
	seek: Option<&str>,
	duration: Option<&str>,
	duration_to: Option<&str>,
) -> Option<Duration> {
	if let Some(t) = duration {
		return parse_ffmpeg_duration(t);
	}

	if let (Some(seek), Some(to)) = (parse_seek(seek), duration_to) {
		return parse_ffmpeg_duration(to).map(|to| to.saturating_sub(seek));
	}

	None
}

/// Generates the correct scale filter based on the given arguments.
pub(crate) fn generate_scale_filter(
	width: Option<u64>,
	height: Option<u64>,
	size_fit: Option<&String>,
	size_fill: Option<&String>,
	scale_mode: ScaleAlgorithm,
) -> Option<Scale> {
	#[allow(clippy::cast_possible_truncation)]
	match (width, height, size_fit, size_fill) {
		(_, _, Some(fit), _) => {
			if let Ok(size_filter) =
				Scale::from_size(fit.clone(), ScaleForceOriginalAspectRatio::Decrease, scale_mode)
			{
				return Some(size_filter);
			}
			None
		}
		(_, _, _, Some(fill)) => {
			if let Ok(size_filter) =
				Scale::from_size(fill.clone(), ScaleForceOriginalAspectRatio::Increase, scale_mode)
			{
				return Some(size_filter);
			}
			None
		}
		(Some(w), Some(h), _, _) => Some(Scale::new(w as i32, h as i32, scale_mode)),
		(Some(w), None, _, _) => Some(Scale::preserve_aspect_ratio_width(w as i32, scale_mode)),
		(None, Some(h), _, _) => Some(Scale::preserve_aspect_ratio_height(h as i32, scale_mode)),
		_ => None,
	}
}

/// Generates a filter chain that ends in a single 16x16 output labeled `palette`.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn palette_to_ffmpeg(pal: &Palette) -> FilterChainList {
	let colors = pal.colors.iter().map(|e| e.color).collect::<Vec<_>>();

	// Create as many filter chains like `color,format[pX] as there are colors in the palette
	let mut color_sources = FilterChainList::new();
	for (color_idx, color) in colors.iter().enumerate() {
		let mut chain = FilterChain::with_outputs([format!("p{}", color_idx + 1)]);
		chain.push(ffmpeg::filters::Color::pixel(color.to_string()));
		chain.push(Format::new("rgb24"));
		color_sources.push(chain);
	}

	// Create "dummy" filter chains repeating the last color of the palette until we hit 256 total filter chains
	let mut dummy_source = FilterChain::new();
	if color_sources.len() < 256 {
		let num_dummies = 256 - color_sources.len();
		let dummy_color = colors.last().unwrap().to_string();

		let dummy_outputs = (0..num_dummies)
			.map(|i| format!("d{}", color_sources.len() + i + 1))
			.collect::<Vec<_>>();
		dummy_source = FilterChain::with_outputs(dummy_outputs);
		dummy_source.push(ffmpeg::filters::Color::pixel(dummy_color));
		dummy_source.push(Format::new("rgb24"));
		dummy_source.push(Split::new(num_dummies as u32));
	}

	// Grab the color filter chains' output names…
	let color_source_outputs = color_sources
		.iter()
		.map(|f| f.outputs.first().unwrap().clone())
		.collect::<Vec<_>>();

	// …and now we have all the output names.
	let all_color_inputs = [color_source_outputs, dummy_source.outputs.clone()].concat();

	// We plug them into a new filter chain that has the single "palette" output
	// and contains an xstack filter that combines all the sources into one 16x16 frame.
	let mut palette_chain = FilterChain::with_inputs_and_outputs(all_color_inputs, ["palette"]);
	palette_chain.push(Xstack::grid(16, 16, None));

	// And now we just return all of the chains in a big Vec!
	let mut all_chains = FilterChainList::new();
	all_chains.extend(color_sources);
	if !dummy_source.is_empty() {
		all_chains.push(dummy_source);
	}
	all_chains.extend([palette_chain]);

	all_chains
}

pub(crate) fn generate_color_sharpness_filters(
	brightness: f64,
	contrast: f64,
	saturation: f64,
	sharpness: f64,
) -> Option<FilterChain> {
	if brightness == 0.0 && contrast == 1.0 && saturation == 1.0 && sharpness == 0.0 {
		return None;
	}

	let mut filters = FilterChain::new();
	filters.push(Eq {
		brightness,
		contrast,
		saturation,
		..Default::default()
	});

	filters.push(Unsharp::new(sharpness));

	if filters.is_empty() { None } else { Some(filters) }
}

/// This is a small wrapper for [ffprobe] that repeats the invocation with frame counting
/// enabled if ffprobe can't find a duration the first time.
pub(crate) fn ffprobe_output<P: AsRef<Path>>(input: P) -> Result<FFProbeOutput> {
	let p = ffprobe(&input, false)?;
	if p.duration().is_ok() {
		return Ok(p);
	}

	#[cfg(debug_assertions)]
	eprintln!("Running ffprobe again and counting frames…");
	ffprobe(&input, true)
}

/// This is a small wrapper for [ffprobe] that repeats the invocation with frame counting
/// enabled if `nb_frames` isn't set the first time.
/// This relies on `nb_frames` being accurate, which might be a problem.
/// We'll simply not worry about it :3
pub(crate) fn ffprobe_frames<P: AsRef<Path>>(input: P) -> Result<FFProbeOutput> {
	let p = ffprobe(&input, false)?;
	if !p.has_video_streams() {
		anyhow::bail!("The input file contains no usable video streams")
	}

	let all_video_streams_have_nb_frames = p
		.streams
		.iter()
		.filter_map(|s| match s.codec_type {
			StreamType::Video => Some(s.nb_frames.is_some()),
			_ => None,
		})
		.all(|x| x);

	if !all_video_streams_have_nb_frames {
		return ffprobe(&input, true);
	}

	Ok(p)
}

pub(crate) fn check_frame_size(w: u64, h: u64) -> Result<()> {
	// adapted from ffmpeg's av_image_check_size2:
	// https://github.com/FFmpeg/FFmpeg/blob/75960ac2708659344bc33b4c108e4a49a0d3184e/libavutil/imgutils.c#L289

	// turns out ffmpeg assesses image size using AV_PIX_FMT_NONE instead of an actual pixel format
	// this feels like an oversight, but I'm not familiar enough with ffmpeg's inner workings to say for sure

	let stride = w * 8 + 128 * 8;
	let stride_area = stride * (h + 128);

	#[cfg(debug_assertions)]
	eprintln!("{w}×{h} → stride: {stride}, stride_area: {stride_area}");

	if w == 0 || h == 0 || w > MAX32 || h > MAX32 || stride >= MAX32 || stride_area >= MAX32 {
		anyhow::bail!("ffmpeg can't handle frames as big as {w}×{h}!")
	}

	Ok(())
}
