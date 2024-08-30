use crate::commands::Cli;
use crate::palettes::{get_builtin_palette, BuiltInPalette};
use crate::vec_push_ext::PushStrExt;
use anyhow::{anyhow, Result};
use ffauto_rs::ffmpeg::enums::{DitherMode, StatsMode};
use ffauto_rs::palettes::palette::{Color, Palette};
use ffauto_rs::timestamps::parse_ffmpeg_timestamp;
use std::fmt::Debug;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn handle_seek<P: AsRef<Path>>(ffmpeg_args: &mut Vec<String>, input: P, seek: &Option<String>) {
	if let Some(ss) = seek {
		ffmpeg_args.push_str("-ss");
		let s = parse_ffmpeg_timestamp(ss).unwrap_or_default().as_secs_f64();
		ffmpeg_args.push(format!("{s}"));
	}

	ffmpeg_args.push_str("-i");
	ffmpeg_args.push(input.as_ref().to_str().unwrap().to_string());
}

pub(crate) fn handle_duration(ffmpeg_args: &mut Vec<String>, seek: &Option<String>, duration: &Option<String>, duration_to: &Option<String>) {
	let mut s = 0_f64;
	if let Some(ss) = seek {
		s = parse_ffmpeg_timestamp(ss).unwrap_or_default().as_secs_f64();
	}

	if let Some(t) = duration {
		match parse_ffmpeg_timestamp(t) {
			Some(t) => {
				ffmpeg_args.push_str("-t");
				ffmpeg_args.push(format!("{}", t.as_secs_f64()));
			}
			None => { eprintln!("invalid duration string: {t}") }
		}
	} else if let Some(to) = duration_to {
		match parse_ffmpeg_timestamp(to) {
			Some(to) => {
				ffmpeg_args.push_str("-t");
				ffmpeg_args.push(format!("{}", to.as_secs_f64() - s));
			}
			None => { eprintln!("invalid duration string: {to}") }
		}
	}
}

pub(crate) fn palette_to_ffmpeg(pal: Palette) -> String {
	let colors = pal.colors.iter().map(|e| e.color).collect::<Vec<Color>>();
	let dummy_color = colors.last().unwrap().to_string();

	let mut color_sources: Vec<String> = Vec::new();
	for (color_idx, color) in colors.iter().enumerate() {
		let source = format!("color=c={color}:r=1:s=1x1,format=rgb24[p{}]", color_idx + 1);
		color_sources.push(source);
	}

	let mut all_sources = (0..color_sources.len()).map(|i| format!("[p{}]", i + 1)).collect::<Vec<String>>().join("");
	if color_sources.len() < 256 {
		let num_dummies = 256 - color_sources.len();
		let all_dummies = (0..num_dummies).map(|i| format!("[dummy{}]", i + 1)).collect::<Vec<String>>().join("");
		let source = format!("color=c={dummy_color}:r=1:s=1x1,format=rgb24,split={num_dummies} {all_dummies}");
		color_sources.push(source);

		all_sources += &*all_dummies;
	}

	color_sources.push(format!("{all_sources}xstack=grid=16x16"));
	color_sources.join(";")
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_full_filtergraph(
	gif: bool,
	video_filter_str: String,
	palette_file: &Option<PathBuf>, palette_name: &Option<BuiltInPalette>,
	num_colors: u16, stats_mode: &StatsMode, diff_rect: bool,
	dither: &DitherMode, bayer_scale: u16,
) -> Result<String> {
	let diff_mode = if diff_rect { "rectangle" } else { "none" };
	let new_palette = if stats_mode == &StatsMode::Single { 1 } else { 0 };

	if palette_file.is_none() && palette_name.is_none() {
		if gif {
			Ok(
				[
					format!("[0:v] {video_filter_str},split [a][b]"),
					format!("[a] palettegen=max_colors={}:reserve_transparent=0:stats_mode={} [pal]", num_colors, stats_mode),
					format!("[b][pal] paletteuse=dither={}:bayer_scale={}:diff_mode={diff_mode}:new={new_palette}", dither, bayer_scale),
				].join(";")
			)
		} else {
			Ok(
				[
					format!("[0:v] {video_filter_str},split [a][b]"),
					format!("[a] palettegen=max_colors={}:reserve_transparent=0 [pal]", num_colors),
					format!("[b][pal] paletteuse=dither={}:bayer_scale={}", dither, bayer_scale),
				].join(";")
			)
		}
	} else {
		let pal: Palette;
		if let Some(palette_file) = palette_file {
			pal = Palette::load_from_file(palette_file).map_err(|e| anyhow!(e))?;
		} else if let Some(palette_name) = palette_name {
			pal = get_builtin_palette(palette_name);
		} else {
			return Err(anyhow!("This wasn't supposed to happen!"));
		}

		let pal_string = palette_to_ffmpeg(pal);

		if gif {
			Ok(
				[
					format!("{pal_string} [pal]"),
					format!("[0:v] {video_filter_str} [filtered];[filtered][pal] paletteuse=dither={}:bayer_scale={}:diff_mode={diff_mode}:new={new_palette}", dither, bayer_scale),
				].join(";")
			)
		} else {
			Ok(
				[
					format!("{pal_string} [pal]"),
					format!("[0:v] {video_filter_str} [filtered];[filtered][pal] paletteuse=dither={}:bayer_scale={}", dither, bayer_scale),
				].join(";")
			)
		}
	}
}

pub(crate) fn generate_scale_filter(cli: &Cli) -> Option<String> {
	if let Some(width) = cli.width {
		return Some(format!("scale=w={width}:h=-2:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", cli.scale_mode));
	} else if let Some(height) = cli.height {
		return Some(format!("scale=w=-2:h={height}:flags={}+accurate_rnd+full_chroma_int+full_chroma_inp", cli.scale_mode));
	}

	None
}

pub(crate) fn debug_pause<D: Debug>(args: D, ffmpeg_args: &[String]) {
	println!("{:#^40}", " DEBUG MODE ");
	println!("program args: {:?}", args);
	println!("ffmpeg args: {}", ffmpeg_args.join(" "));
	let mut stdout = io::stdout();
	let stdin = io::stdin();
	write!(stdout, "{:#^40}", " Press Enter to continue… ").unwrap();
	stdout.flush().unwrap();
	let _ = stdin.read_line(&mut "".to_string()).unwrap();
	writeln!(stdout, "Continuing…").unwrap();
}