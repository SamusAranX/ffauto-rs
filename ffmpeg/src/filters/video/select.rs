use ffmpeg_macro::filter;

/// Select video frames to pass in output.
///
/// If the expression is evaluated to zero, the frame is discarded. If the evaluation result is
/// negative or NaN, the frame is sent to the first output; otherwise it is sent to the output with
/// index ceil(val)-1, assuming that the input index starts from 0.
#[filter(name = "select")]
pub struct Select {
	/// Set expression, which is evaluated for each input frame.
	#[ffarg(noname)]
	pub expr: String,

	/// Set the number of outputs. The output to which to send the selected frame is based on the
	/// result of the evaluation.
	#[ffarg(name = "outputs", default = 1, omit_default)]
	pub outputs: u32,
}

impl Select {
	pub fn new<S: Into<String>>(expr: S, outputs: u32) -> Self {
		Self {
			expr: expr.into(),
			outputs
		}
	}
}

#[test]
fn filter_select() {
	let filter = Select::default();
	assert_eq!(filter.to_string(), "select");
}

#[test]
fn filter_select_params() {
	let filter = Select::new("eq(n,0)", 1);
	assert_eq!(filter.to_string(), "select=eq(n,0)");
}