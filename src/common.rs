use std::fmt::Debug;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use ffauto_rs::ffmpeg::enums::{Crop, DitherMode, StatsMode};
use ffauto_rs::ffmpeg::sizes::parse_ffmpeg_size;
use ffauto_rs::ffmpeg::timestamps::parse_ffmpeg_timestamp;
use ffauto_rs::palettes::palette::{Color, Palette};

use crate::commands::Cli;
use crate::palettes::{get_builtin_palette, BuiltInPalette};
use crate::vec_push_ext::PushStrExt;

/// Parses the seek option, then inserts the seek and input parameters into the ffmpeg_args vector.
/// Returns the parsed seek value.
pub(crate) fn handle_seek<P: AsRef<Path>>(ffmpeg_args: &mut Vec<String>, input: P, seek: &Option<String>) -> f64 {
	let mut s = 0_f64;
	if let Some(ss) = seek {
		ffmpeg_args.push_str("-ss");
		s = parse_ffmpeg_timestamp(ss).unwrap_or_default().as_secs_f64();
		ffmpeg_args.push(format!("{s}"));
	}

	ffmpeg_args.push_str("-i");
	ffmpeg_args.push(input.as_ref().to_str().unwrap().to_string());

	s
}

/// Parses the duration, then inserts an appropriate -t <value> into the ffmpeg_args vector.
/// Returns the parsed duration value.
pub(crate) fn handle_duration(ffmpeg_args: &mut Vec<String>, seek: f64, duration: &Option<String>, duration_to: &Option<String>) -> f64 {
	let mut dur = 0_f64;

	if let Some(t) = duration {
		match parse_ffmpeg_timestamp(t) {
			Some(t) => {
				dur = t.as_secs_f64();
				ffmpeg_args.push_str("-t");
				ffmpeg_args.push(format!("{}", dur));
			}
			None => { eprintln!("invalid duration string: {t}") }
		}
	} else if let Some(to) = duration_to {
		match parse_ffmpeg_timestamp(to) {
			Some(to) => {
				dur = to.as_secs_f64() - seek;
				ffmpeg_args.push_str("-t");
				ffmpeg_args.push(format!("{}", dur));
			}
			None => { eprintln!("invalid duration string: {to}") }
		}
	}

	dur
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

pub(crate) fn generate_scale_filter(cli: &Cli) -> Result<String> {
	if let Some(width) = cli.width {
		return Ok(format!("scale=w={width}:h=-2:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", cli.scale_mode));
	} else if let Some(height) = cli.height {
		return Ok(format!("scale=w=-2:h={height}:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", cli.scale_mode));
	} else if let Some(size) = &cli.size {
		let size = parse_ffmpeg_size(size)?;
		return Ok(
			format!("scale=w={}:h={}:force_original_aspect_ratio=decrease:force_divisible_by=2:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp",
			        size.width, size.height, cli.scale_mode)
		);
	}

	Ok("".parse().unwrap())
}

pub(crate) fn add_basic_filters(video_filter: &mut Vec<String>, cli: &Cli, color_transfer: String) -> Result<()> {
	if let Some(crop_str) = &cli.crop {
		let crop = Crop::new(crop_str)?;
		video_filter.push(format!("crop={crop}"));
	}

	let scale = generate_scale_filter(cli)?;
	if !scale.is_empty() {
		video_filter.push(scale);
	}

	if color_transfer.contains("smpte2084") || color_transfer.contains("arib-std-b67") {
		video_filter.push_str("zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709");
	}

	Ok(())
}

pub(crate) fn add_palette_filters(video_filter: &mut Vec<String>, brightness: f64, contrast: f64, saturation: f64, sharpness: f64) {
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

		video_filter.push(format!("eq={}", eq_args.join(":")));
	}

	if sharpness != 0.0 {
		video_filter.push(format!("unsharp=la={0}:ca={0}", sharpness));
	}
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_palette_filtergraph(
	gif: bool, dedup: bool,
	video_filter_str: String,
	palette_file: &Option<PathBuf>, palette_name: &Option<BuiltInPalette>,
	num_colors: u16, stats_mode: &StatsMode, diff_rect: bool,
	dither: &DitherMode, bayer_scale: u16,
) -> Result<String> {
	let mpdecimate = if gif && dedup { ",mpdecimate" } else { "" };
	let bayer_scale = if dither == &DitherMode::Bayer { format!(":bayer_scale={bayer_scale}") } else { String::new() };
	let diff_mode = if gif && diff_rect { ":diff_mode=rectangle" } else { "" };
	let new = if gif && stats_mode == &StatsMode::Single { ":new=1" } else { "" };

	if palette_file.is_some() || palette_name.is_some() {
		let pal_string: String;
		if let Some(palette_file) = palette_file {
			let pal = Palette::load_from_file(palette_file).map_err(|e| anyhow!(e))?;
			pal_string = palette_to_ffmpeg(pal);
		} else if let Some(palette_name) = palette_name {
			let pal = get_builtin_palette(palette_name);
			pal_string = palette_to_ffmpeg(pal);
		} else {
			return Err(anyhow!("This wasn't supposed to happen!"));
		}

		Ok(
			[
				format!("{pal_string} [pal]"),
				format!("[0:v] {video_filter_str}{mpdecimate},setsar=1 [filtered];[filtered][pal] paletteuse=dither={dither}{bayer_scale}{diff_mode}"),
			].join(";")
		)
	} else {
		// no palette was given, so we'll use palettegen to create one
		let stats_mode = if gif { format!(":stats_mode={stats_mode}") } else { String::new() };

		Ok(
			[
				format!("[0:v] {video_filter_str}{mpdecimate},setsar=1,split [a][b]"),
				format!("[a] palettegen=max_colors={num_colors}:reserve_transparent=0{stats_mode} [pal]"),
				format!("[b][pal] paletteuse=dither={dither}{bayer_scale}{diff_mode}{new}"),
			].join(";")
		)
	}
}

pub(crate) fn debug_pause<D: Debug>(args: D, ffmpeg_args: &[String]) {
	println!("{:#^40}", " DEBUG MODE ");
	println!("program args: {:?}", args);

	let ffmpeg_args = ffmpeg_args
		.iter().map(|a| if a.contains(" ") { format!("\"{a}\"") } else { a.to_owned() })
		.collect::<Vec<String>>();

	println!("full command: ffmpeg {}", ffmpeg_args.join(" "));
	let mut stdout = io::stdout();
	let stdin = io::stdin();
	write!(stdout, "{:#^40}", " Press Enter to continue… ").unwrap();
	stdout.flush().unwrap();
	let _ = stdin.read_line(&mut "".to_string()).unwrap();
	writeln!(stdout, "Continuing…").unwrap();
}