use ffmpeg_macro::filter;

/// Stack video inputs into a custom layout. All streams must be of the same pixel format.
#[filter(name = "xstack")]
pub struct Xstack {
	/// Set number of input streams. Default is 2.
	#[ffarg(default = 2, omit_default)]
	pub inputs: u32,

	/// Specify layout of inputs. Each input is separated by '|'. The position of each input is
	/// specified as two values separated by '_', representing the x (column) and y (row) offset.
	/// Optionally, wX and hX can be used to reference the width or height of input X. Multiple
	/// values can be summed together with '+'. Example: `0_0|w0_0|0_h0|w0_h0` for a 2x2 grid.
	#[ffarg(omit_default)]
	pub layout: String,

	/// Specify a fixed grid layout as COLUMNSxROWS. When set, the inputs option is ignored and
	/// is implicitly set to rows*columns.
	#[ffarg(omit_default)]
	pub grid: String,

	/// If set to 1, force the output to terminate when the shortest input terminates.
	#[ffarg(omit_default)]
	pub shortest: bool,

	/// Specify the fill color for any unused areas of the output frame.
	pub fill: Option<String>,
}

impl Xstack {
	#[must_use]
	pub fn grid(columns: u64, rows: u64, fill: Option<String>) -> Self {
		Self {
			grid: format!("{columns}x{rows}"),
			fill,
			..Default::default()
		}
	}
}

#[test]
fn filter_xstack() {
	let filter = Xstack::default();
	assert_eq!(filter.to_string(), "xstack");
}

#[test]
fn filter_xstack_grid() {
	let filter = Xstack::grid(16, 16, Some("black".to_string()));
	assert_eq!(filter.to_string(), "xstack=grid=16x16:fill=black");
}
