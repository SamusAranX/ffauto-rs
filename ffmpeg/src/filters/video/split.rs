use ffmpeg_macro::filter;

/// Split video input into several identical outputs.
/// If the number of outputs is unspecified, it defaults to 2.
#[filter(name = "split")]
pub struct Split {
	/// Specifies the number of outputs.
	#[ffarg(noname, default = 2)]
	pub outputs: u32,
}

impl Split {
	pub fn new(outputs: u32) -> Self {
		Self { outputs }
	}
}

#[test]
fn filter_split() {
	let filter = Split::default();
	assert_eq!(filter.to_string(), "split=2");
}

#[test]
fn filter_split_params() {
	let filter = Split::new(42);
	assert_eq!(filter.to_string(), "split=42");
}