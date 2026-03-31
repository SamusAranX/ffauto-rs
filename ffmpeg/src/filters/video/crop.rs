use anyhow::anyhow;
use ffmpeg_macro::filter;
use regex::Regex;

/// Crop the input video to given dimensions.
#[filter(name = "crop")]
pub struct Crop {
	/// The width of the output video. This expression is evaluated only once during the filter
	/// configuration, or when the 'w' or 'out_w' command is sent.
	#[ffarg(name = "out_w", omit_default)]
	pub width: i32,

	/// The height of the output video. This expression is evaluated only once during the filter
	/// configuration, or when the 'h' or 'out_h' command is sent.
	#[ffarg(name = "out_h")]
	pub height: i32,

	/// The horizontal position, in the input video, of the left edge of the output video.
	/// This expression is evaluated per-frame.
	pub x: Option<i32>,

	/// The vertical position, in the input video, of the top edge of the output video.
	/// This expression is evaluated per-frame.
	pub y: Option<i32>,

	/// If set to `true` will force the output display aspect ratio to be the same of the input,
	/// by changing the output sample aspect ratio.
	#[ffarg(default = false, omit_default)]
	pub keep_aspect: bool,

	/// Enable exact cropping. If enabled, subsampled videos will be cropped at exact
	/// width/height/x/y as specified and will not be rounded to nearest smaller value.
	#[ffarg(default = false, omit_default)]
	pub exact: bool,
}

impl Crop {
	#[must_use]
	pub fn new(width: i32, height: i32, x: i32, y: i32) -> Self {
		Self {
			width,
			height,
			x: Some(x),
			y: Some(y),
			..Default::default()
		}
	}

	#[must_use]
	pub fn new_only_size(width: i32, height: i32) -> Self {
		Self { width, height, ..Default::default() }
	}

	pub fn from_arg<S: Into<String>>(crop_arg: S) -> anyhow::Result<Self> {
		let crop_str = crop_arg.into();
		let re = Regex::new(r"(-?\d+)").unwrap();

		let numbers = re
			.find_iter(crop_str.as_str())
			.map(|s| {
				s.as_str()
					.parse::<i32>()
					.map_err(|_| anyhow!("\"{crop_str}\" is not a valid crop value"))
			})
			.collect::<anyhow::Result<Vec<_>, anyhow::Error>>()?;

		match numbers.as_slice() {
			[h] if *h > 0 => Ok(Crop { height: *h, ..Crop::default() }),
			[w, h] if *w > 0 && *h > 0 => Ok(Crop::new_only_size(*w, *h)),
			[w, h, x, y] if *w > 0 && *h > 0 => Ok(Crop::new(*w, *h, *x, *y)),
			_ => anyhow::bail!("\"{crop_str}\" is not a valid crop value"),
		}
	}
}

#[test]
fn filter_crop() {
	let filter = Crop::default();
	assert_eq!(filter.to_string(), "crop=out_w=0:out_h=0");
}

#[test]
fn filter_crop_params() {
	let filter = Crop {
		width: 800,
		height: 480,
		x: Some(560),
		y: Some(300),
		keep_aspect: true,
		exact: true,
	};
	assert_eq!(
		filter.to_string(),
		"crop=out_w=800:out_h=480:x=560:y=300:keep_aspect=1:exact=1"
	);
}
