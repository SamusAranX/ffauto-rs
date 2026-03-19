use ffmpeg::filters::{Fade, Palettegen};

#[test]
fn filter_fade() {
	let filter = Fade::default();
	assert_eq!(filter.to_string(), "fade=type=in:start_frame=0:nb_frames=25:alpha=0:start_time=0:duration=0:color=black");
}

#[test]
fn filter_palettegen() {
	let filter = Palettegen::default();
	assert_eq!(filter.to_string(), "palettegen=max_colors=256:reserve_transparent=1:transparency_color=lime:stats_mode=full");
}
