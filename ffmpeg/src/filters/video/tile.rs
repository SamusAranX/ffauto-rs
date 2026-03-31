use ffmpeg_macro::filter;

/// Tile several successive frames together. The untile filter can do the reverse.
#[filter(name = "tile")]
pub struct Tile {
	/// Set the grid size in the form COLUMNSxROWS. Range is up to UINT_MAX cells.
	#[ffarg(default = "6x5")]
	pub layout: String,

	/// Set the maximum number of frames to render in the given area. It must be less than or equal
	/// to wxh. A value of 0 means all the area will be used.
	#[ffarg(omit_default)]
	pub nb_frames: u64,

	/// Set the outer border margin in pixels. Range is 0 to 1024.
	#[ffarg(omit_default)]
	pub margin: u16,

	/// Set the inner border thickness in pixels (i.e. the number of pixels between frames). Range
	/// is 0 to 1024.
	#[ffarg(omit_default)]
	pub padding: u16,

	/// Specify the color of the unused area.
	#[ffarg(default = "black", omit_default)]
	pub color: String,
}

impl Tile {
	#[must_use]
	pub fn columns(num: u64) -> Self {
		Self {
			layout: format!("{num}x1"),
			nb_frames: num,
			..Default::default()
		}
	}

	#[must_use]
	pub fn rows(num: u64) -> Self {
		Self {
			layout: format!("1x{num}"),
			nb_frames: num,
			..Default::default()
		}
	}
}
