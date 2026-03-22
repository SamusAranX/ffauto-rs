use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum PaletteuseDither {
	/// Ordered 8x8 bayer dithering (deterministic).
	#[strum(serialize = "bayer")]
	Bayer,

	/// Dithering as defined by Paul Heckbert in 1982 (simple error diffusion).
	/// Note: this dithering is sometimes considered "wrong" and is included as a reference.
	#[strum(serialize = "heckbert")]
	Heckbert,

	/// Floyd and Steinberg dithering (error diffusion).
	#[strum(serialize = "floyd_steinberg")]
	FloydSteinberg,

	/// Frankie Sierra dithering v2 (error diffusion).
	#[strum(serialize = "sierra2")]
	Sierra2,

	/// Frankie Sierra dithering v2 "Lite" (error diffusion).
	#[strum(serialize = "sierra2_4a")]
	#[default]
	Sierra2_4a,

	/// Frankie Sierra dithering v3 (error diffusion).
	#[strum(serialize = "sierra3")]
	Sierra3,

	/// Burkes dithering (error diffusion).
	#[strum(serialize = "burkes")]
	Burkes,

	/// Atkinson dithering by Bill Atkinson at Apple Computer (error diffusion).
	#[strum(serialize = "atkinson")]
	Atkinson,

	/// Disable dithering.
	#[strum(serialize = "none")]
	None,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum PaletteuseDiffMode {
	#[strum(serialize = "none")]
	#[default]
	None,

	/// Only the changing rectangle will be reprocessed. This is similar to GIF
	/// cropping/offsetting compression mechanism. This option can be useful for speed if only a
	/// part of the image is changing, and has use cases such as limiting the scope of the error
	/// diffusion dither to the rectangle that bounds the moving scene (it leads to more
	/// deterministic output if the scene doesn't change much, and as a result less moving noise
	/// and better GIF compression).
	#[strum(serialize = "rectangle")]
	Rectangle,
}

/// Use a palette to downsample an input video stream.
#[filter(name = "paletteuse")]
pub struct Paletteuse {
	/// Select dithering mode.
	#[ffarg()]
	pub dither: PaletteuseDither,

	/// When bayer dithering is selected, this option defines the scale of the pattern (how much
	/// the crosshatch pattern is visible). A low value means more visible pattern for less
	/// banding, and a higher value means less visible pattern at the cost of more banding. Must be
	/// an integer value in the range [0, 5].
	#[ffarg(default = 2, omit_default)]
	pub bayer_scale: u8,

	/// If set, defines the zone to process.
	#[ffarg(omit_default)]
	pub diff_mode: PaletteuseDiffMode,

	/// Take new palette for each output frame.
	#[ffarg(omit_default)]
	pub new: bool,

	/// Sets the alpha threshold for transparency. Alpha values above this threshold will be
	/// treated as completely opaque, and values below this threshold will be treated as completely
	/// transparent. Must be an integer value in the range [0, 255].
	#[ffarg(default = 128, omit_default)]
	pub alpha_threshold: u8,
}