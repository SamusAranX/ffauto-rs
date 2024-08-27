use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;
use const_format::formatcp;
use ffauto_rs::ffmpeg_enums::{DitherMode, Preset, ScaleMode, StatsMode, VideoCodec};
use std::path::PathBuf;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const GIT_VERSION: &str = env!("GIT_VERSION");
const BUILD_DATE: &str = env!("BUILD_DATE");

const CLAP_VERSION: &str = formatcp!("{GIT_VERSION} [{GIT_BRANCH}, {GIT_HASH}, {BUILD_DATE}]");

#[derive(Parser, Debug)]
#[command(version = CLAP_VERSION, about = "Wraps common ffmpeg workflows")]
pub struct Cli {
	#[command(subcommand)]
	pub command: Option<Commands>,

	#[arg(long = "ss", global = true, help = "The start time offset")]
	pub seek: Option<String>,

	#[arg(short, long, global = true, help = "Crops the output video. Format WxH or WxH,X;Y. (applied before scaling)")]
	pub crop: Option<String>,

	#[arg(long = "vw", group = "resize", global = true, help = "Sets the output video width, preserving aspect ratio.")]
	pub width: Option<u64>,
	#[arg(long = "vh", group = "resize", global = true, help = "Sets the output video height, preserving aspect ratio.")]
	pub height: Option<u64>,
	#[arg(short = 'S', long, global = true, value_enum, help = "Scaling algorithm", default_value_t = ScaleMode::default())]
	pub scale_mode: ScaleMode,

	#[arg(long, global = true)]
	pub debug: bool,
}

#[derive(Parser, Debug)]
pub struct AutoArgs {
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

	#[arg(short, long, help = "Sets the fade in and out durations. Takes precedence over -fi/-fo.", default_value_t = 0.0)]
	pub fade: f64,
	#[arg(long = "fi", help = "Sets the fade in duration.", default_value_t = 0.0)]
	pub fade_in: f64,
	#[arg(long = "fo", help = "Sets the fade out duration.", default_value_t = 0.0)]
	pub fade_out: f64,

	#[arg(short = 'r', long, help = "Sets the output video frame rate.")]
	pub framerate: Option<f64>,

	#[arg(short = 'C', long = "codec", help = "The video codec", default_value_t = VideoCodec::default())]
	pub video_codec: VideoCodec,
	#[arg(short, long, help = "The encoder preset", default_value_t = Preset::default())]
	pub preset: Preset,

	#[arg(short, help = "Reduces video quality depending on how often this was specified", action = ArgAction::Count)]
	pub garbage: u8,
}

impl AutoArgs {
	pub fn audio_copy_possible(&self, input_codec_name: Option<String>) -> bool {
		!self.mute && input_codec_name == Some("aac".parse().unwrap()) && self.audio_volume == 1.0 &&
			self.fade == 0.0 && self.fade_in == 0.0 && self.fade_out == 0.0
	}

	pub fn needs_audio_filter(&self) -> bool {
		self.audio_volume != 1.0 || self.fade != 0.0 || self.fade_in != 0.0 || self.fade_out != 0.0
	}

	pub fn needs_video_filter(&self, cli: &Cli) -> bool {
		cli.width.is_some() || cli.height.is_some() || self.fade != 0.0 || self.fade_in != 0.0 || self.fade_out != 0.0 ||
			cli.crop.is_some() || self.framerate.is_some() || self.tonemap
	}
}

#[derive(Parser, Debug)]
pub struct GIFArgs {
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

	#[arg(short = 'r', long, help = "Sets the output video frame rate.")]
	pub framerate: Option<f64>,

	#[arg(short = 'n', help = "The number of colors in the palette (palettegen)", default_value_t = 256)]
	pub num_colors: u16,
	#[arg(long, help = "The statistics mode (palettegen)", default_value_t = StatsMode::default())]
	pub stats_mode: StatsMode, // StatsMode::Single implies paletteuse:new

	#[arg(long, help = "Affects the output brightness, range [-1.0;1.0]", default_value_t = 0.0)]
	pub brightness: f64,
	#[arg(long, help = "Affects the output contrast, range [-1000.0;1000.0]", default_value_t = 1.0)]
	pub contrast: f64,
	#[arg(long, help = "Affects the output saturation, range [0.0;3.0]", default_value_t = 1.0)]
	pub saturation: f64,
	#[arg(long, help = "Affects the output sharpness, range [-1.5;1.5]", default_value_t = 0.0)]
	pub sharpness: f64,

	#[arg(short = 'D', long, help = "The dithering mode (paletteuse)", default_value_t = DitherMode::default())]
	pub dither: DitherMode,
	#[arg(long, help = "The bayer pattern scale in the range [0;5] (paletteuse)", default_value_t = 2)]
	pub bayer_scale: u16,
	#[arg(long, help = "Only reprocess the changed rectangle (Helps with noise and compression)")]
	pub diff_rect: bool,
}

#[derive(Parser, Debug)]
pub struct QuantizeArgs {
	#[arg(short, help = "The input file")]
	pub input: PathBuf,
	#[arg(help = "The output file")]
	pub output: PathBuf,

	#[arg(short = 'n', help = "The number of colors in the palette (palettegen)", default_value_t = 256)]
	pub num_colors: u16,

	#[arg(short = 'D', long, help = "The dithering mode (paletteuse)", default_value_t = DitherMode::default())]
	pub dither: DitherMode,
	#[arg(long, help = "The bayer pattern scale in the range [0;5] (paletteuse)", default_value_t = 2)]
	pub bayer_scale: u16,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	#[command(about = "Common ffmpeg wrapper")]
	Auto(AutoArgs),

	#[command(about = "GIF encoder with a subset of features")]
	Gif(GIFArgs),

	#[command(about = "Uses ffmpeg to quantize still images")]
	Quantize(QuantizeArgs),
}