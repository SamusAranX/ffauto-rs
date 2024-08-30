use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FFProbeOutput {
	pub streams: Vec<Stream>,
}

#[derive(clap::ValueEnum, Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
	Audio,
	Video,
	Subtitle,
	Data,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Stream {
	pub index: u64,
	pub codec_name: Option<String>,
	pub codec_type: StreamType,
	pub width: Option<u64>,
	pub height: Option<u64>,
	pub pix_fmt: Option<String>,
	pub color_range: Option<String>,
	pub color_space: Option<String>,
	pub color_transfer: Option<String>,
	pub color_primaries: Option<String>,
	pub r_frame_rate: Option<String>,
	pub avg_frame_rate: Option<String>,
	pub sample_rate: Option<String>,
	pub channels: Option<u64>,
	pub bit_rate: Option<String>,
	pub duration: Option<String>,
	pub nb_read_frames: Option<u64>,
}

impl Stream {
	pub fn frame_rate(&self) -> Option<f64> {
		match &self.r_frame_rate {
			None => { return None; }
			Some(fps) => {
				if fps.contains("/") {
					if let Some(split) = fps.split_once("/") {
						let left = split.0.parse::<f64>().unwrap();
						let right = split.1.parse::<f64>().unwrap();
						return Some(left / right);
					}
				} else {
					return fps.parse::<f64>().ok();
				}
			}
		}

		None
	}
}