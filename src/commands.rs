use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;
use const_format::formatcp;
use std::path::PathBuf;

use crate::palettes::BuiltInPalette;
use ffauto_rs::ffmpeg::enums::{BarcodeMode, DitherMode, OptimizeTarget, ScaleMode, StatsMode, VideoCodec};

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

	#[arg(long, global = true)]
	pub debug: bool,
}

#[allow(unreachable_code)]
fn default_accelerator() -> String {
	#[cfg(target_os = "macos")]
	return "videotoolbox".to_string();

	"auto".to_string()
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct AutoArgs {
	/// The input file.
	#[arg(short)]
	pub input: PathBuf,
	/// The output file.
	#[arg()]
	pub output: PathBuf,

	/// Selects video streams by index or ISO 639-2 language code.
	#[arg(long, alias = "Vs", default_values_t = ["0".to_string()])]
	pub video_streams: Vec<String>,
	/// Selects audio streams by index or ISO 639-2 language code.
	#[arg(long, alias = "As", default_values_t = ["0".to_string()])]
	pub audio_streams: Vec<String>,
	/// Selects subtitle streams by index or ISO 639-2 language code.
	#[arg(long, alias = "Ss")]
	pub sub_streams: Vec<String>,

	/// The start time offset.
	#[arg(short = 's', long)]
	pub seek: Option<String>,

	/// The output duration.
	#[arg(short = 't', group = "seeking")]
	pub duration: Option<String>,
	#[arg(long = "to", group = "seeking")]
	pub duration_to: Option<String>,

	/// Crops the output video. Format H, WxH, or WxH,X;Y. (applied before scaling)
	#[arg(short, long)]
	pub crop: Option<String>,

	/// Sets the output video width, preserving aspect ratio.
	#[arg(long = "vw", group = "resize")]
	pub width: Option<u64>,
	/// Sets the output video height, preserving aspect ratio.
	#[arg(long = "vh", group = "resize")]
	pub height: Option<u64>,
	/// Sets the rectangle the output video size must fit into. Format WxH or an ffmpeg size name.
	#[arg(long = "vs", group = "resize")]
	pub size: Option<String>,
	/// Sets the scaling algorithm used.
	#[arg(short = 'S', long, value_enum, default_value_t = ScaleMode::default())]
	pub scale_mode: ScaleMode,

	/// Performs an HDR-to-SDR tonemap.
	#[arg(short = 'T', long)]
	pub tonemap: bool,
	/// Moves moov atom to the start. (Enabled by default, use -F=false to disable)
	#[arg(short = 'F', long, default_value_t = true)]
	pub faststart: bool,
	/// Experimental: Enables hardware-assisted decoding. Might break things.
	#[arg(short = 'H', long)]
	pub hwaccel: bool,
	/// Used with --hwaccel. Defaults to "videotoolbox" on macOS and "auto" everywhere else.
	#[arg(short = 'a', long, default_value_t = default_accelerator())]
	pub accelerator: String,

	/// Removes the audio stream.
	#[arg(short = 'M', long, group = "volume")]
	pub mute: bool,
	/// Sets the output audio volume factor.
	#[arg(short = 'v', long = "volume", group = "volume", default_value_t = 1.0)]
	pub audio_volume: f64,

	/// Sets the number of output audio channels.
	#[arg(long = "channels")]
	pub audio_channels: Option<String>,

	/// Sets the fade in and out durations. Takes precedence over --fi/--fo.
	#[arg(short, long, default_value_t = 0.0)]
	pub fade: f64,
	/// Sets the fade in duration.
	#[arg(long = "fi", default_value_t = 0.0)]
	pub fade_in: f64,
	/// Sets the fade out duration.
	#[arg(long = "fo", default_value_t = 0.0)]
	pub fade_out: f64,

	/// Sets the output video frame rate.
	#[arg(short = 'r', long, group = "framerates")]
	pub framerate: Option<f64>,
	/// Sets the output video frame rate to a factor of the input video frame rate.
	#[arg(short = 'R', long, group = "framerates")]
	pub framerate_mult: Option<f64>,

	/// The output video codec.
	#[arg(short = 'C', long = "codec", default_value_t = VideoCodec::default())]
	pub video_codec: VideoCodec,

	/// Optimizes settings for certain devices.
	#[arg(short = 'O', long = "optimize")]
	pub optimize_target: Option<OptimizeTarget>,

	/// Increasingly reduces video quality (in turn reducing output file size) depending on how often this was specified.
	#[arg(short,
	action = ArgAction::Count)]
	pub garbage: u8,
}

impl AutoArgs {
	pub(crate) fn audio_copy_possible(&self, input_codec_name: Option<String>) -> bool {
		!self.mute
			&& self.audio_channels.is_none()
			&& input_codec_name == Some("aac".parse().unwrap())
			&& self.audio_volume == 1.0
			&& self.fade <= 0.0
			&& self.fade_in <= 0.0
			&& self.fade_out <= 0.0
	}

	pub(crate) fn needs_audio_filter(&self) -> bool {
		self.audio_volume != 1.0 || self.fade > 0.0 || self.fade_in > 0.0 || self.fade_out > 0.0
	}

	pub(crate) fn needs_video_filter(&self) -> bool {
		self.width.is_some()
			|| self.height.is_some()
			|| self.size.is_some()
			|| self.fade > 0.0
			|| self.fade_in > 0.0
			|| self.fade_out > 0.0
			|| self.crop.is_some()
			|| self.framerate.is_some()
			|| self.tonemap
	}

	pub(crate) fn optimize_settings(&mut self) {
		match self.optimize_target {
			None => return,
			_ => {
				self.width = None;
				self.height = None;
				self.size = None;
				self.tonemap = true; // none of the optimization targets support HDR media
				self.faststart = true;
				self.audio_channels = Some("2".parse().unwrap());
				self.video_codec = VideoCodec::H264;
			}
		}

		match self.optimize_target {
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
pub(crate) struct BarcodeArgs {
	/// The input file.
	#[arg(short)]
	pub input: PathBuf,
	/// The output file. (always outputs PNG)
	#[arg()]
	pub output: PathBuf,

	/// Selects a video stream by index.
	#[arg(long, group = "video_select", default_value_t = 0)]
	pub video_stream: usize,
	/// Selects a video stream by language. (ISO 639-2)
	#[arg(long = "video-lang", group = "video_select")]
	pub video_language: Option<String>,

	/// Override the number of frames, skipping ffprobe's potentially lengthy frame counting process.
	#[arg(short = 'f', long = "frames")]
	pub video_frames: Option<u64>,

	/// The barcode strip generation method.
	#[arg(short = 'B', long, value_enum, default_value_t = BarcodeMode::default())]
	pub barcode_mode: BarcodeMode,
	/// Outputs a 48-bit PNG.
	#[arg(short = 'D', long)]
	pub deep_color: bool,

	/// Sets the output barcode image's height.
	#[arg(long = "vh", group = "resize")]
	pub height: Option<u64>,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct GIFArgs {
	/// The input file.
	#[arg(short)]
	pub input: PathBuf,
	/// The output file.
	#[arg()]
	pub output: PathBuf,

	/// Selects a video stream by index.
	#[arg(long, group = "video_select", default_value_t = 0)]
	pub video_stream: usize,
	/// Selects a video stream by language. (ISO 639-2)
	#[arg(long = "video-lang", group = "video_select")]
	pub video_language: Option<String>,

	/// The start time offset.
	#[arg(short = 's', long)]
	pub seek: Option<String>,

	/// The output duration.
	#[arg(short = 't', group = "seeking")]
	pub duration: Option<String>,
	/// The end time offset.
	#[arg(long = "to", group = "seeking")]
	pub duration_to: Option<String>,

	/// Crops the output video. Format H, WxH, or WxH,X;Y. (applied before scaling)
	#[arg(short, long)]
	pub crop: Option<String>,

	/// Sets the output video width, preserving aspect ratio.
	#[arg(long = "vw", group = "resize")]
	pub width: Option<u64>,
	/// Sets the output video height, preserving aspect ratio.
	#[arg(long = "vh", group = "resize")]
	pub height: Option<u64>,
	/// Sets the rectangle the output video size must fit into. Format WxH or an ffmpeg size name.
	#[arg(long = "vs", group = "resize")]
	pub size: Option<String>,
	/// Sets the scaling algorithm used.
	#[arg(short = 'S', long, value_enum, default_value_t = ScaleMode::default())]
	pub scale_mode: ScaleMode,

	/// Sets the fade in and out durations. Takes precedence over --fi/--fo.
	#[arg(short, long, default_value_t = 0.0)]
	pub fade: f64,
	/// Sets the fade in duration.
	#[arg(long = "fi", default_value_t = 0.0)]
	pub fade_in: f64,
	/// Sets the fade out duration.
	#[arg(long = "fo", default_value_t = 0.0)]
	pub fade_out: f64,

	/// Sets the output video frame rate.
	#[arg(short = 'r', long, group = "framerates")]
	pub framerate: Option<f64>,
	/// Sets the output video frame rate to a factor of the input video frame rate.
	#[arg(short = 'R', long, group = "framerates")]
	pub framerate_mult: Option<f64>,

	/// Attempts to deduplicate frames.
	#[arg(long)]
	pub dedup: bool,

	/// Affects the output brightness, range [-1.0;1.0]
	#[arg(long, allow_negative_numbers = true, default_value_t = 0.0)]
	pub brightness: f64,
	/// Affects the output contrast, range [-1000.0;1000.0]
	#[arg(long, allow_negative_numbers = true, default_value_t = 1.0)]
	pub contrast: f64,
	/// Affects the output saturation, range [0.0;3.0]
	#[arg(long, default_value_t = 1.0)]
	pub saturation: f64,
	/// Affects the output sharpness, range [-1.5;1.5]
	#[arg(long, allow_negative_numbers = true, default_value_t = 0.0)]
	pub sharpness: f64,

	/// A file containing a palette. (supports ACT, COL, GPL, HEX, and PAL formats)
	#[arg(short, long, group = "palette")]
	pub palette_file: Option<PathBuf>,
	/// A built-in palette.
	#[arg(short = 'P', long, group = "palette")]
	pub palette_name: Option<BuiltInPalette>,
	/// The number of colors in the generated palette.
	#[arg(short = 'n', group = "palette", default_value_t = 256)]
	pub num_colors: u16,

	/// The statistics mode. (palettegen)
	#[arg(long, default_value_t = StatsMode::default())]
	pub stats_mode: StatsMode, // StatsMode::Single implies paletteuse:new

	/// The dithering mode. (paletteuse)
	#[arg(short = 'D', long, default_value_t = DitherMode::default())]
	pub dither: DitherMode,
	/// The bayer pattern scale in the range [0;5] (paletteuse)
	#[arg(long, default_value_t = 2)]
	pub bayer_scale: u8,
	/// Only reprocess the changed rectangle. (Helps with noise and compression) (paletteuse)
	#[arg(long)]
	pub diff_rect: bool,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct QuantArgs {
	/// The input file.
	#[arg(short)]
	pub input: PathBuf,
	/// The output file.
	#[arg()]
	pub output: PathBuf,

	/// Selects a video stream by index.
	#[arg(long, group = "video_select", default_value_t = 0)]
	pub video_stream: usize,
	/// Selects a video stream by language. (ISO 639-2)
	#[arg(long = "video-lang", group = "video_select")]
	pub video_language: Option<String>,

	/// The start time offset.
	#[arg(short = 's', long)]
	pub seek: Option<String>,

	/// Crops the output video. Format H, WxH, or WxH,X;Y. (applied before scaling)
	#[arg(short, long)]
	pub crop: Option<String>,

	/// Sets the output video width, preserving aspect ratio.
	#[arg(long = "vw", group = "resize")]
	pub width: Option<u64>,
	/// Sets the output video height, preserving aspect ratio.
	#[arg(long = "vh", group = "resize")]
	pub height: Option<u64>,
	/// Sets the rectangle the output video size must fit into. Format WxH or an ffmpeg size name.
	#[arg(long = "vs", group = "resize")]
	pub size: Option<String>,
	/// Sets the scaling algorithm used.
	#[arg(short = 'S', long, value_enum, default_value_t = ScaleMode::default())]
	pub scale_mode: ScaleMode,

	/// Affects the output brightness, range [-1.0;1.0]
	#[arg(long, allow_negative_numbers = true, default_value_t = 0.0)]
	pub brightness: f64,
	/// Affects the output contrast, range [-1000.0;1000.0]
	#[arg(long, allow_negative_numbers = true, default_value_t = 1.0)]
	pub contrast: f64,
	/// Affects the output saturation, range [0.0;3.0]
	#[arg(long, default_value_t = 1.0)]
	pub saturation: f64,
	/// Affects the output sharpness, range [-1.5;1.5]
	#[arg(long, allow_negative_numbers = true, default_value_t = 0.0)]
	pub sharpness: f64,

	/// A file containing a palette in either ACT, COL, GPL, HEX, JSON, or PAL format.
	#[arg(short, long, group = "palette")]
	pub palette_file: Option<PathBuf>,
	/// A built-in palette.
	#[arg(short = 'P', long, group = "palette")]
	pub palette_name: Option<BuiltInPalette>,
	/// The number of colors in the generated palette
	#[arg(short = 'n', group = "palette", default_value_t = 256)]
	pub num_colors: u16,

	/// The dithering mode (paletteuse)
	#[arg(short = 'D', long, default_value_t = DitherMode::default())]
	pub dither: DitherMode,
	/// The bayer pattern scale in the range [0;5] (paletteuse)
	#[arg(long, default_value_t = 2)]
	pub bayer_scale: u8,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct InfoArgs {
	/// The input file.
	#[arg(short)]
	pub input: PathBuf,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
	#[command(about = "Common ffmpeg wrapper")]
	Auto(AutoArgs),

	#[command(about = "Movie barcode generator")]
	Barcode(BarcodeArgs),

	#[command(about = "GIF encoder with a subset of features")]
	Gif(GIFArgs),

	#[command(about = "Uses ffmpeg to quantize still images")]
	Quant(QuantArgs),

	#[command(about = "Formats and prints ffprobe information")]
	Info(InfoArgs),
}
