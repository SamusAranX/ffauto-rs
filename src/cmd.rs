use crate::vec_push_ext::PushStrExt;
use ffauto_rs::timestamps::parse_ffmpeg_timestamp;
use std::path::Path;

pub fn handle_seek_and_duration<P: AsRef<Path>>(ffmpeg_args: &mut Vec<String>, input: P, seek: &Option<String>, duration: &Option<String>, duration_to: &Option<String>) {
	let mut s = 0_f64;
	if let Some(ss) = seek {
		ffmpeg_args.push_str("-ss");
		s = parse_ffmpeg_timestamp(ss).unwrap_or_default().as_secs_f64();
		ffmpeg_args.push(format!("{s}"));
	}

	ffmpeg_args.push_str("-i");
	ffmpeg_args.push(input.as_ref().to_str().unwrap().to_string());

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