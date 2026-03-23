use ffmpeg::filters::{Crop, Fade, Palettegen, Scale, Tonemap, TonemapAlgorithm};

#[test]
fn filter_fade() {
	let filter = Fade::default();
	assert_eq!(filter.to_string(), "fade=type=in:start_frame=0:nb_frames=25");
}

#[test]
fn filter_palettegen() {
	let filter = Palettegen::default();
	assert_eq!(filter.to_string(), "palettegen=max_colors=256");
}

#[test]
fn filter_scale() {
	let filter = Scale::default();
	assert_eq!(filter.to_string(), "scale=w=0:h=0:flags=accurate_rnd+full_chroma_int+full_chroma_inp");
}

#[test]
fn filter_crop() {
	let filter = Crop::default();
	assert_eq!(filter.to_string(), "crop=out_w=0:out_h=0:x=0:y=0");
}

#[test]
fn filter_tonemap() {
	let filter = Tonemap::default();
	assert_eq!(filter.to_string(), "tonemap=tonemap=none");
}

#[test]
fn filter_tonemap_params() {
	let filter = Tonemap {
		algorithm: TonemapAlgorithm::Reinhard,
		desat: 5.0,
		peak: 1000.0,
	};
	assert_eq!(filter.to_string(), "tonemap=tonemap=reinhard:desat=5:peak=1000");
}