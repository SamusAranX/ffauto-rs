use clap::ArgAction;
use std::path::PathBuf;

use clap::Parser;
use const_format::formatcp;

use ffauto_rs::ffmpeg_enums::*;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const GIT_VERSION: &str = env!("GIT_VERSION");
const BUILD_DATE: &str = env!("BUILD_DATE");

const CLAP_VERSION: &str = formatcp!("{GIT_VERSION} [{GIT_BRANCH}, {GIT_HASH}, {BUILD_DATE}]");

#[derive(Parser, Debug)]
#[command(version = CLAP_VERSION, about = "Wraps common simple ffmpeg workflows")]
pub struct ProgramArgs {
	#[clap(short, help = "The input file")]
	pub input: PathBuf,

	#[clap(long = "ss", help = "The start time offset")]
	pub seek: Option<String>,

	#[clap(short = 't', group = "seeking", help = "The output duration")]
	pub duration: Option<String>,
	#[clap(long = "to", group = "seeking", help = "The end time offset")]
	pub seek_to: Option<String>,

	#[clap(short = 'M', long, group = "volume", help = "Removes the audio stream")]
	pub mute: bool,
	#[clap(short = 'v', long = "volume", group = "volume", help = "Sets the output audio volume factor", default_value_t = 1.0)]
	pub audio_volume: f64,

	#[clap(short = 'W', group = "resize", help = "Sets the output video width, preserving aspect ratio (unless height is also specified).")]
	pub width: Option<u64>,
	#[clap(short = 'H', group = "resize", help = "Sets the output video height, preserving aspect ratio (unless width is also specified).")]
	pub height: Option<u64>,
	#[clap(short = 'S', long, value_enum, help = "Scaling algorithm", default_value_t = ScaleMode::default())]
	pub scale_mode: ScaleMode,

	#[clap(short, long, help = "Sets the fade in and out durations. Takes precedence over -fi/-fo.", default_value_t=0.0)]
	pub fade: f64,
	#[clap(long = "fi", help = "Sets the fade in duration.", default_value_t=0.0)]
	pub fade_in: f64,
	#[clap(long = "fo", help = "Sets the fade out duration.", default_value_t=0.0)]
	pub fade_out: f64,

	#[clap(short, long, help = "Crops the output video. Format WxH or WxH,X;Y.")]
	pub crop: Option<String>,
	// #[clap(short, long, num_args(2..4), help = "Crops the output video. Takes values W H or X Y W H.")]
	// pub crop: Option<Vec<u16>>,

	#[clap(short = 'r', long, help = "Sets the output video frame rate.")]
	pub framerate: Option<f64>,

	#[clap(long = "c:v", help = "The video codec", default_value_t = VideoCodec::default())]
	pub video_codec: VideoCodec,
	#[clap(short, long, help = "The encoder preset", default_value_t = Preset::default())]
	pub preset: Preset,

	#[clap(short, help = "Reduces video quality according to how often this was specified", action = ArgAction::Count)]
	pub garbage: u8,

	#[clap(short='F', long, help = "Moves moov atom to the start")]
	pub faststart: bool,

	#[clap(help = "The output file")]
	pub output: PathBuf,

	#[clap(long)]
	pub debug: bool,
}

impl ProgramArgs {
	pub fn audio_copy_possible(&self, input_codec_name: Option<String>) -> bool {
		if self.mute {
			return false;
		}

		if input_codec_name != Some("aac".parse().unwrap()) {
			return false;
		}

		if self.audio_volume != 1.0 {
			return false;
		}

		if self.fade != 0.0 || self.fade_in != 0.0 || self.fade_out != 0.0 {
			return false;
		}

		true
	}

	pub fn needs_audio_filter(&self) -> bool {
		self.audio_volume != 1.0 || self.fade != 0.0 || self.fade_in != 0.0 || self.fade_out != 0.0
	}

	pub fn needs_video_filter(&self) -> bool {
		self.width.is_some() || self.height.is_some() || self.fade != 0.0 || self.fade_in != 0.0 || self.fade_out != 0.0 ||
			self.crop.is_some() || self.framerate.is_some() || self.garbage > 0
	}
}