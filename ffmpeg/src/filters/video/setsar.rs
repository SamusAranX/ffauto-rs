use ffmpeg_macro::filter;

/// Sets the Sample (aka Pixel) Aspect Ratio for the filter output video.
///
/// Note that as a consequence of the application of this filter, the output display aspect ratio
/// will change according to the equation DAR = HORIZONTAL_RESOLUTION / VERTICAL_RESOLUTION * SAR.
///
/// Keep in mind that the sample aspect ratio set by the setsar filter may be changed by later
/// filters in the filterchain, e.g. if another "setsar" or a "setdar" filter is applied.
#[filter(name = "setsar")]
pub struct SetSar {
	/// Set the aspect ratio used by the filter. The parameter can be a floating point number
	/// string, or an expression. If the parameter is not specified, the value "0" is assumed,
	/// meaning that the same input value is used.
	#[ffarg(noname, omit_default)]
	pub sar: f64,
}

impl SetSar {
	pub fn new(sar: f64) -> Self {
		Self {
			sar
		}
	}
}

#[test]
fn filter_setsar() {
	let filter = SetSar::default();
	assert_eq!(filter.to_string(), "setsar");
}

#[test]
fn filter_setsar_params() {
	let filter = SetSar::new(1.0);
	assert_eq!(filter.to_string(), "setsar=1");
}