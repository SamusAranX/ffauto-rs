use ffmpeg_macro::filter;

/// Set output format constraints for the input audio.
/// The framework will negotiate the most appropriate format to minimize conversions.
#[filter(name = "aformat")]
pub struct Aformat {
	/// A list of requested sample formats.
	#[ffarg(name = "sample_fmts", separator = "|")]
	pub sample_formats: Vec<String>,

	/// A list of requested sample rates.
	#[ffarg(separator = "|")]
	pub sample_rates: Vec<String>,

	/// A list of requested channel layouts.
	#[ffarg(separator = "|")]
	pub channel_layouts: Vec<String>,
}

impl Aformat {
	/// Constructs a filter with the set of channel layouts AAC supports on its own.
	/// Sample format and rate are left unconstrained.
	#[must_use]
	pub fn aac_channel_layouts() -> Self {
		Self {
			channel_layouts: ["mono", "stereo", "3.0", "4.0", "5.0", "5.1", "7.1"]
				.into_iter()
				.map(String::from)
				.collect(),
			..Default::default()
		}
	}
}

#[test]
fn filter_aformat() {
	let filter = Aformat::default();

	assert_eq!(filter.to_string(), "aformat");
}

#[test]
fn filter_aformat_aac_channel_layouts() {
	let filter = Aformat::aac_channel_layouts();

	assert_eq!(
		filter.to_string(),
		"aformat=channel_layouts=mono|stereo|3.0|4.0|5.0|5.1|7.1"
	);
}

#[test]
fn filter_aformat_params() {
	let filter = Aformat {
		sample_formats: vec!["fltp".into()],
		sample_rates: vec!["48000".into()],
		channel_layouts: vec!["stereo".into()],
	};

	assert_eq!(
		filter.to_string(),
		"aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo"
	);
}
