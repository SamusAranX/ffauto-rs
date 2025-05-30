use crate::palettes::{BuiltInPalette, get_builtin_palette};
use anyhow::Result;
use ffauto_rs::ffmpeg::enums::{Crop, DitherMode, ScaleMode, StatsMode};
use ffauto_rs::ffmpeg::ffprobe::ffprobe;
use ffauto_rs::ffmpeg::ffprobe_struct::{FFProbeOutput, StreamType};
use ffauto_rs::ffmpeg::sizes::parse_ffmpeg_size;
use ffauto_rs::ffmpeg::timestamps::parse_ffmpeg_duration;
use ffauto_rs::palettes::palette::{Color, Palette};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX32: u64 = i32::MAX as u64;

pub(crate) const TONEMAP_FILTER: &str = "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709";
pub(crate) const SCALE_FLAGS: &str = "accurate_rnd+full_chroma_int+full_chroma_inp";

pub trait CanSeek {
	fn parse_seek(&self) -> Option<Duration>;
}

pub trait CanSetDuration {
	fn parse_duration(&self) -> Option<Duration>;
}

pub trait CanCrop {
	fn generate_crop_filter(&self) -> Option<String>;
}

pub trait CanScale {
	fn generate_scale_filter(&self) -> Option<String>;
}

pub trait CanChangeFPS {
	fn generate_fps_filter(&self, stream_fps: Option<f64>) -> Option<String>;
}

pub trait CanColorFilter {
	fn generate_color_filters(&self) -> Option<String>;
}

pub trait CanGeneratePalette {
	fn generate_palette_filters(&self) -> Result<String>;
}

/// Parses the seek string and returns it as a [Duration], if present.
pub(crate) fn parse_seek(seek: &Option<String>) -> Option<Duration> {
	if let Some(seek_str) = seek {
		return parse_ffmpeg_duration(seek_str);
	}

	None
}

/// Parses the duration strings and returns an appropriate [Duration].
pub(crate) fn parse_duration(seek: &Option<String>, duration: &Option<String>, duration_to: &Option<String>) -> Option<Duration> {
	if let Some(t) = duration {
		return parse_ffmpeg_duration(t);
	}

	if let (Some(seek), Some(to)) = (parse_seek(seek), duration_to) {
		return parse_ffmpeg_duration(to).map(|to| to - seek);
	}

	None
}

pub(crate) fn palette_to_ffmpeg(pal: Palette) -> String {
	let colors = pal.colors.iter().map(|e| e.color).collect::<Vec<Color>>();

	let mut color_sources: Vec<String> = Vec::new();
	for (color_idx, color) in colors.iter().enumerate() {
		let source = format!("color=c={color}:r=1:s=1x1,format=rgb24[p{}]", color_idx + 1);
		color_sources.push(source);
	}

	let mut all_sources = (0..color_sources.len()).map(|i| format!("[p{}]", i + 1)).collect::<Vec<String>>().join("");
	if color_sources.len() < 256 {
		let num_dummies = 256 - color_sources.len();
		let all_dummies = (0..num_dummies).map(|i| format!("[d{}]", i + 1)).collect::<Vec<String>>().join("");
		let dummy_color = colors.last().unwrap().to_string();
		let source = format!("color=c={dummy_color}:r=1:s=1x1,format=rgb24,split={num_dummies} {all_dummies}");
		color_sources.push(source);

		all_sources += &*all_dummies;
	}

	color_sources.push(format!("{all_sources}xstack=grid=16x16"));
	color_sources.join(";")
}

pub(crate) fn generate_crop_filter(crop: &Option<String>) -> Option<String> {
	if let Some(crop_str) = crop {
		return Crop::new(crop_str).map(|c| format!("crop={c}")).ok();
	}

	None
}

pub(crate) fn generate_scale_filter(width: Option<u64>, height: Option<u64>, size: &Option<String>, scale_mode: &ScaleMode) -> Option<String> {
	if let Some(width) = width {
		return Some(format!("scale=w={width}:h=-2:flags={}+{SCALE_FLAGS}", scale_mode));
	} else if let Some(height) = height {
		return Some(format!("scale=w=-2:h={height}:flags={}+{SCALE_FLAGS}", scale_mode));
	} else if let Some(size) = size {
		return match parse_ffmpeg_size(size) {
			Ok(size) => Some(format!(
				"scale=w={}:h={}:force_original_aspect_ratio=decrease:force_divisible_by=2:flags={}+{SCALE_FLAGS}",
				size.width, size.height, scale_mode
			)),
			Err(err) => {
				eprintln!("{err}");
				None
			}
		};
	}

	None
}

pub(crate) fn generate_fps_filter(fps_arg: Option<f64>, fps_mult_arg: Option<f64>, stream_fps: Option<f64>) -> Option<String> {
	if let Some(fps) = fps_arg {
		Some(format!("fps=fps={fps:.3}"));
	} else if let (Some(fps_mult), Some(fps)) = (fps_mult_arg, stream_fps) {
		Some(format!("fps=fps={:.3}", fps * fps_mult));
	}

	None
}

pub(crate) fn generate_color_sharpness_filters(brightness: f64, contrast: f64, saturation: f64, sharpness: f64) -> Option<String> {
	let mut filter_parts: Vec<String> = vec![];

	if brightness != 0.0 || contrast != 1.0 || saturation != 1.0 {
		let mut eq_args: Vec<String> = vec![];

		if brightness != 0.0 {
			eq_args.push(format!("brightness={}", brightness))
		}
		if contrast != 1.0 {
			eq_args.push(format!("contrast={}", contrast))
		}
		if saturation != 1.0 {
			eq_args.push(format!("saturation={}", saturation))
		}

		filter_parts.push(format!("eq={}", eq_args.join(":")));
	}

	if sharpness != 0.0 {
		filter_parts.push(format!("unsharp=la={0}:ca={0}", sharpness));
	}

	if filter_parts.is_empty() {
		None
	} else {
		Some(filter_parts.join(","))
	}
}

#[allow(clippy::too_many_arguments)]
/// This function generates a chain of filters that should be appended to the very end of a filtergraph.
pub(crate) fn generate_palette_filtergraph(
	palette_file: &Option<PathBuf>,
	palette_name: &Option<BuiltInPalette>,
	num_colors: u16,
	stats_mode: &StatsMode,
	diff_rect: bool,
	dither: &DitherMode,
	bayer_scale: u8,
) -> Result<String> {
	fn palette_filter_string(pal_string: String, paletteuse_args: String) -> String {
		[
			",setsar=1 [filtered]".to_string(),
			format!("{pal_string} [pal]"),
			format!("[filtered][pal] paletteuse={paletteuse_args}"),
		]
		.join(";")
	}

	let paletteuse_args = {
		let mut args = HashMap::new();
		args.insert("dither".to_string(), format!("{dither}"));
		if dither == &DitherMode::Bayer {
			args.insert("bayer_scale".to_string(), format!("{bayer_scale}"));
		}
		if diff_rect {
			args.insert("diff_mode".to_string(), "rectangle".to_string());
		}
		if palette_file.is_none() && palette_name.is_none() {
			let new = (stats_mode == &StatsMode::Single) as u8;
			args.insert("new".to_string(), format!("{new}"));
		}

		args
			.into_iter()
			.map(|(k, v)| format!("{k}={v}"))
			.collect::<Vec<String>>()
			.join(":")
	};

	match (palette_file, palette_name) {
		(Some(palette_file), None) => {
			#[allow(clippy::needless_late_init)]
			let pal_string: String;
			match Palette::load_from_file(palette_file) {
				Ok(pal) => pal_string = palette_to_ffmpeg(pal),
				Err(e) => anyhow::bail!(e),
			}
			Ok(palette_filter_string(pal_string, paletteuse_args))
		}
		(None, Some(palette_name)) => {
			let pal_string = palette_to_ffmpeg(get_builtin_palette(palette_name));
			Ok(palette_filter_string(pal_string, paletteuse_args))
		}
		(None, None) => {
			// no palette was given, so we'll use palettegen to create one
			Ok([
				",setsar=1,split [a][b]".to_string(),
				format!("[a] palettegen=max_colors={num_colors}:reserve_transparent=0:stats_mode={stats_mode} [pal]"),
				format!("[b][pal] paletteuse={paletteuse_args}"),
			]
			.join(";"))
		}
		_ => anyhow::bail!("Well, this wasn't supposed to happen."),
	}
}

/// This is a small wrapper for [ffprobe] that repeats the invocation with frame counting
/// enabled if ffprobe can't find a duration the first time.
pub(crate) fn ffprobe_output<P: AsRef<Path>>(input: P) -> Result<FFProbeOutput> {
	let p = ffprobe(&input, false)?;
	match p.duration() {
		Ok(_) => Ok(p),
		Err(_) => {
			#[cfg(debug_assertions)]
			eprintln!("Running ffprobe again and counting frames…");
			ffprobe(&input, true)
		}
	}
}

/// This is a small wrapper for [ffprobe] that repeats the invocation with frame counting
/// enabled if nb_frames isn't set the first time.
/// This relies on nb_frames being accurate, which might be a problem.
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
	eprintln!("stride: {stride} | stride_area: {stride_area}");

	if w == 0 || h == 0 || w > MAX32 || h > MAX32 || stride >= MAX32 || stride_area >= MAX32 {
		anyhow::bail!("ffmpeg can't handle frames as big as {w}×{h}!")
	}

	Ok(())
}
