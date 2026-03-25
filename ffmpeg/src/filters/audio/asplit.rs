use ffmpeg_macro::filter;

/// Split audio input into several identical outputs.
/// If the number of outputs is unspecified, it defaults to 2.
#[filter(name = "asplit")]
pub struct Asplit {
	/// Specifies the number of outputs.
	#[ffarg(noname, default = 2)]
	pub outputs: u32,
}

impl Asplit {
	#[must_use]
	pub fn new(outputs: u32) -> Self {
		Self { outputs }
	}
}

#[test]
fn filter_asplit() {
	let filter = Asplit::default();
	assert_eq!(filter.to_string(), "asplit=2");
}

#[test]
fn filter_asplit_params() {
	let filter = Asplit::new(42);
	assert_eq!(filter.to_string(), "asplit=42");
}
