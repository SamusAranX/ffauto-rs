use std::path::PathBuf;

use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;
use const_format::formatcp;

use ffauto_rs::ffmpeg::enums::{DitherMode, OptimizeTarget, ScaleMode, StatsMode, VideoCodec};

use crate::palettes::BuiltInPalette;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const GIT_VERSION: &str = env!("GIT_VERSION");
const BUILD_DATE: &str = env!("BUILD_DATE");

const CLAP_VERSION: &str = formatcp!("{GIT_VERSION} [{GIT_BRANCH}, {GIT_HASH}, {BUILD_DATE}]");

#[derive(Parser, Debug, Clone)]
#[command(version = CLAP_VERSION, about = "Wraps common ffmpeg workflows")]
pub(crate) struct Cli {
	#[command(subcommand)]
	pub command: Option<Commands>,

	#[arg(short = 's', long, global = true, help = "The start time offset")]
	pub seek: Option<String>,

	#[arg(short, long, global = true, help = "Crops the output video. Format H, WxH, or WxH,X;Y. (applied before scaling)")]
	pub crop: Option<String>,

	#[arg(long = "vw", group = "resize", global = true, help = "Sets the output video width, preserving aspect ratio.")]
	pub width: Option<u64>,
	#[arg(long = "vh", group = "resize", global = true, help = "Sets the output video height, preserving aspect ratio.")]
	pub height: Option<u64>,
	#[arg(long = "vs", group = "resize", global = true, help = "Sets the rectangle the output video size must fit into. Format WxH or an ffmpeg size name.")]
	pub size: Option<String>,
	#[arg(short = 'S', long, global = true, value_enum, help = "Scaling algorithm", default_value_t = ScaleMode::default())]
	pub scale_mode: ScaleMode,

	#[arg(long, global = true)]
	pub debug: bool,
}

impl Cli {
	pub(crate) fn optimize_settings(&mut self, optimize_target: &Option<OptimizeTarget>) {
		match optimize_target {
			None => (),
			Some(_) => {
				self.width = None;
				self.height = None;
				self.size = None;
			}
		}

		match optimize_target {
			None => (),
			Some(OptimizeTarget::Ipod5) => {
				self.size = Some("320x240".to_string());
			}
			Some(OptimizeTarget::Ipod) => {
				self.size = Some("640x480".to_string());
			}
			Some(OptimizeTarget::Psp) => {
				// as of firmware 3.30, allegedly supports MPEG-4 AVC Main Profile 720x480, 352x480 and 480x272
				// extra info: also supports 160x120 JPEG thumbnails with a .THM extension, next to the video files
				self.size = Some("480x272".to_string());
			}
			Some(OptimizeTarget::PsVita) => {
				self.size = Some("960x540".to_string());
			}
		}
	}
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct AutoArgs {
	#[arg(short, help = "The input file")]
	pub input: PathBuf,
	#[arg(help = "The output file")]
	pub output: PathBuf,

	#[arg(short = 't', group = "seeking", help = "The output duration")]
	pub duration: Option<String>,
	#[arg(long = "to", group = "seeking", help = "The end time offset")]
	pub duration_to: Option<String>,

	#[arg(short = 'T', long, help = "Performs an HDR-to-SDR tonemap")]
	pub tonemap: bool,
	#[arg(short = 'F', long, default_value_t = true, help = "Moves moov atom to the start")]
	pub faststart: bool,

	#[arg(short = 'M', long, group = "volume", help = "Removes the audio stream")]
	pub mute: bool,
	#[arg(short = 'v', long = "volume", group = "volume", help = "Sets the output audio volume factor", default_value_t = 1.0)]
	pub audio_volume: f64,

	#[arg(long = "channels", help = "Sets the number of output audio channels")]
	pub audio_channels: Option<String>,

	#[arg(long, group = "video_select", help = "Selects a video stream by index", default_value_t = 0)]
	pub video_index: u8,
	#[arg(long = "video-lang", group = "video_select", help = "Selects a video stream by language (ISO 639-2)")]
	pub video_language: Option<String>,
	#[arg(long, group = "audio_select", help = "Selects an audio stream by index", default_value_t = 0)]
	pub audio_index: u8,
	#[arg(long = "audio-lang", group = "audio_select", help = "Selects an audio stream by language (ISO 639-2)")]
	pub audio_language: Option<String>,
	#[arg(long, group = "sub_select", help = "Selects a subtitle stream by index")]
	pub sub_index: Option<u8>,
	#[arg(long = "sub-lang", group = "sub_select", help = "Selects a subtitle stream by language (ISO 639-2)")]
	pub sub_language: Option<String>,

	#[arg(short, long, help = "Sets the fade in and out durations. Takes precedence over -fi/-fo.", default_value_t = 0.0)]
	pub fade: f64,
	#[arg(long = "fi", help = "Sets the fade in duration.", default_value_t = 0.0)]
	pub fade_in: f64,
	#[arg(long = "fo", help = "Sets the fade out duration.", default_value_t = 0.0)]
	pub fade_out: f64,

	#[arg(short = 'r', long, group = "framerates", help = "Sets the output video frame rate.")]
	pub framerate: Option<f64>,
	#[arg(short = 'R', long, group = "framerates", help = "Sets the output video frame rate to a factor of the input video frame rate.")]
	pub framerate_mult: Option<f64>,

	#[arg(short = 'C', long = "codec", help = "The video codec", default_value_t = VideoCodec::default())]
	pub video_codec: VideoCodec,

	#[arg(short = 'O', long = "optimize", help = "Optimize settings for certain devices")]
	pub optimize_target: Option<OptimizeTarget>,

	#[arg(short, help = "Reduces video quality depending on how often this was specified", action = ArgAction::Count)]
	pub garbage: u8,
}

impl AutoArgs {
	pub(crate) fn audio_copy_possible(&self, input_codec_name: Option<String>) -> bool {
		!self.mute
			&& self.audio_channels.is_none()
			&& input_codec_name == Some("aac".parse().unwrap())
			&& self.audio_volume == 1.0
			&& self.fade == 0.0
			&& self.fade_in == 0.0
			&& self.fade_out == 0.0
	}

	pub(crate) fn needs_audio_filter(&self) -> bool {
		self.audio_volume != 1.0 || self.fade != 0.0 || self.fade_in != 0.0 || self.fade_out != 0.0
	}

	pub(crate) fn needs_video_filter(&self, cli: &Cli) -> bool {
		cli.width.is_some()
			|| cli.height.is_some()
			|| cli.size.is_some()
			|| self.fade != 0.0
			|| self.fade_in != 0.0
			|| self.fade_out != 0.0
			|| cli.crop.is_some()
			|| self.framerate.is_some()
			|| self.tonemap
	}

	pub(crate) fn optimize_settings(&mut self) {
		match self.optimize_target {
			None => (),
			_ => {
				self.tonemap = true; // none of the optimization targets support HDR media
				self.faststart = true;
				self.audio_channels = Some("2".parse().unwrap());
				self.video_codec = VideoCodec::H264;
			}
		}
	}
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct GIFArgs {
	#[arg(short, help = "The input file")]
	pub input: PathBuf,
	#[arg(help = "The output file")]
	pub output: PathBuf,

	#[arg(short = 't', group = "seeking", help = "The output duration")]
	pub duration: Option<String>,
	#[arg(long = "to", group = "seeking", help = "The end time offset")]
	pub duration_to: Option<String>,

	#[arg(short, long, help = "Sets the fade in and out durations. Takes precedence over -fi/-fo.", default_value_t = 0.0)]
	pub fade: f64,
	#[arg(long = "fi", help = "Sets the fade in duration.", default_value_t = 0.0)]
	pub fade_in: f64,
	#[arg(long = "fo", help = "Sets the fade out duration.", default_value_t = 0.0)]
	pub fade_out: f64,

	#[arg(short = 'r', long, group = "framerates", help = "Sets the output video frame rate.")]
	pub framerate: Option<f64>,
	#[arg(short = 'R', long, group = "framerates", help = "Sets the output video frame rate to a factor of the input video frame rate.")]
	pub framerate_mult: Option<f64>,

	#[arg(long, help = "Attempts to deduplicate frames.")]
	pub dedup: bool,

	#[arg(long, help = "Affects the output brightness, range [-1.0;1.0]", allow_negative_numbers = true, default_value_t = 0.0)]
	pub brightness: f64,
	#[arg(long, help = "Affects the output contrast, range [-1000.0;1000.0]", allow_negative_numbers = true, default_value_t = 1.0)]
	pub contrast: f64,
	#[arg(long, help = "Affects the output saturation, range [0.0;3.0]", default_value_t = 1.0)]
	pub saturation: f64,
	#[arg(long, help = "Affects the output sharpness, range [-1.5;1.5]", allow_negative_numbers = true, default_value_t = 0.0)]
	pub sharpness: f64,

	#[arg(short, long, group = "palette", help = "A file containing a palette (supports ACT, COL, GPL, HEX, and PAL formats)")]
	pub palette_file: Option<PathBuf>,
	#[arg(short = 'P', long, group = "palette", help = "A built-in palette")]
	pub palette_name: Option<BuiltInPalette>,
	#[arg(short = 'n', group = "palette", help = "The number of colors in the palette (palettegen)", default_value_t = 256)]
	pub num_colors: u16,

	#[arg(long, help = "The statistics mode (palettegen)", default_value_t = StatsMode::default())]
	pub stats_mode: StatsMode, // StatsMode::Single implies paletteuse:new

	#[arg(short = 'D', long, help = "The dithering mode (paletteuse)", default_value_t = DitherMode::default())]
	pub dither: DitherMode,
	#[arg(long, help = "The bayer pattern scale in the range [0;5] (paletteuse)", default_value_t = 2)]
	pub bayer_scale: u16,
	#[arg(long, help = "Only reprocess the changed rectangle (Helps with noise and compression) (paletteuse)")]
	pub diff_rect: bool,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct QuantArgs {
	#[arg(short, help = "The input file")]
	pub input: PathBuf,
	#[arg(help = "The output file")]
	pub output: PathBuf,

	#[arg(long, help = "Affects the output brightness, range [-1.0;1.0]", allow_negative_numbers = true, default_value_t = 0.0)]
	pub brightness: f64,
	#[arg(long, help = "Affects the output contrast, range [-1000.0;1000.0]", allow_negative_numbers = true, default_value_t = 1.0)]
	pub contrast: f64,
	#[arg(long, help = "Affects the output saturation, range [0.0;3.0]", default_value_t = 1.0)]
	pub saturation: f64,
	#[arg(long, help = "Affects the output sharpness, range [-1.5;1.5]", allow_negative_numbers = true, default_value_t = 0.0)]
	pub sharpness: f64,

	#[arg(short, long, group = "palette", help = "A file containing a palette in either ACT, COL, GPL, HEX, JSON, or PAL format")]
	pub palette_file: Option<PathBuf>,
	#[arg(short = 'P', long, group = "palette", help = "A built-in palette")]
	pub palette_name: Option<BuiltInPalette>,
	#[arg(short = 'n', group = "palette", help = "The number of colors in the palette (palettegen)", default_value_t = 256)]
	pub num_colors: u16,

	#[arg(short = 'D', long, help = "The dithering mode (paletteuse)", default_value_t = DitherMode::default())]
	pub dither: DitherMode,
	#[arg(long, help = "The bayer pattern scale in the range [0;5] (paletteuse)", default_value_t = 2)]
	pub bayer_scale: u16,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct InfoArgs {
	#[arg(short, help = "The input file")]
	pub input: PathBuf,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
	#[command(about = "Common ffmpeg wrapper")]
	Auto(AutoArgs),

	#[command(about = "GIF encoder with a subset of features")]
	Gif(GIFArgs),

	#[command(about = "Uses ffmpeg to quantize still images")]
	Quant(QuantArgs),

	#[command(about = "Formats and prints ffprobe information")]
	Info(InfoArgs),
}
