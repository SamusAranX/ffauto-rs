use crate::commands::{AutoArgs, GIFArgs, QuantArgs};
use crate::common::*;
use ffmpeg::filters::FilterChain;
use std::time::Duration;

impl CanSeek for AutoArgs {
	fn parse_seek(&self) -> Option<Duration> {
		parse_seek(self.seek.as_deref())
	}
}

impl CanSetDuration for AutoArgs {
	fn parse_duration(&self) -> Option<Duration> {
		parse_duration(
			self.seek.as_deref(),
			self.duration.as_deref(),
			self.duration_to.as_deref(),
		)
	}
}

impl CanSeek for GIFArgs {
	fn parse_seek(&self) -> Option<Duration> {
		parse_seek(self.seek.as_deref())
	}
}

impl CanSetDuration for GIFArgs {
	fn parse_duration(&self) -> Option<Duration> {
		parse_duration(
			self.seek.as_deref(),
			self.duration.as_deref(),
			self.duration_to.as_deref(),
		)
	}
}

impl CanColorFilter for GIFArgs {
	fn generate_color_filters(&self) -> Option<FilterChain> {
		generate_color_sharpness_filters(self.brightness, self.contrast, self.saturation, self.sharpness)
	}
}

impl CanSeek for QuantArgs {
	fn parse_seek(&self) -> Option<Duration> {
		parse_seek(self.seek.as_deref())
	}
}

impl CanColorFilter for QuantArgs {
	fn generate_color_filters(&self) -> Option<FilterChain> {
		generate_color_sharpness_filters(self.brightness, self.contrast, self.saturation, self.sharpness)
	}
}
