use crate::vec_push_ext::PushStrExt;
use ffauto_rs::palettes::palette::{Color, Palette};
use ffauto_rs::timestamps::parse_ffmpeg_timestamp;
use std::path::Path;

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