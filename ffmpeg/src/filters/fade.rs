use crate::filters::FFmpegFilter;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FadeType {
	#[default]
	In,
	Out,
}

impl Display for FadeType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FadeType::In => write!(f, "in"),
			FadeType::Out => write!(f, "out"),
		}
	}
}

/// Apply a fade-in/out effect to the input video.
#[derive(Debug, Clone)]
pub struct Fade {
	/// The effect type can be either `FadeType::In` for a fade-in, or `FadeType::Out` for a fade-out effect.
	/// Default is `FadeType::In`.
	pub r#type: FadeType,

	/// Specify the number of the frame to start applying the fade effect at.
	/// Default is 0.
	pub start_frame: u32,

	/// The number of frames that the fade effect lasts.
	/// At the end of the fade-in effect, the output video will have the same intensity as the input video.
	/// At the end of the fade-out transition, the output video will be filled with the selected color.
	/// Default is 25.
	pub nb_frames: u32,

	/// If set to true, fade only alpha channel, if one exists on the input.
	/// Default value is false.
	pub alpha: bool,

	/// Specify the timestamp (in seconds) of the frame to start to apply the fade effect.
	/// If both `start_frame` and `start_time` are specified, the fade will start at whichever comes last.
	/// Default is 0.
	pub start_time: f64,

	/// The number of seconds for which the fade effect has to last.
	/// At the end of the fade-in effect the output video will have the same intensity as the input video,
	/// at the end of the fade-out transition the output video will be filled with the selected color.
	/// If both `duration` and `nb_frames` are specified, `duration` is used.
	/// Default is 0 (`nb_frames` is used by default).
	pub duration: f64,

	/// Specify the color of the fade.
	/// Default is "black".
	pub color: String,
}

impl Default for Fade {
	fn default() -> Self {
		Self {
			r#type: FadeType::In,
			start_frame: 0,
			nb_frames: 25,
			alpha: false,
			start_time: 0.0,
			duration: 0.0,
			color: "black".to_string(),
		}
	}
}

impl FFmpegFilter for Fade {
	const NAME: &str = "fade";
}

impl Display for Fade {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let output: Vec<String> = vec![
			format!("{}={}", "type", self.r#type.to_string()),
			format!("{}={}", "start_frame", self.start_frame.to_string()),
			format!("{}={}", "nb_frames", self.nb_frames.to_string()),
			format!("{}={}", "alpha", self.alpha.to_string()),
			format!("{}={}", "start_time", self.start_time.to_string()),
			format!("{}={}", "duration", self.duration.to_string()),
			format!("{}={}", "color", self.color.clone()),
		];

		write!(f, "{}={}", Self::NAME, output.join(":"))
	}
}
