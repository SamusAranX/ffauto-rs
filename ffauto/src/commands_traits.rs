use crate::commands::{AutoArgs, GIFArgs, QuantArgs};
use crate::common::*;
use anyhow::Result;
use ffmpeg::ffmpeg::enums::StatsMode;
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

impl CanCrop for AutoArgs {
	fn generate_crop_filter(&self) -> Option<String> {
		generate_crop_filter(self.crop.as_deref())
	}
}

impl CanScale for AutoArgs {
	fn generate_scale_filter(&self) -> Option<String> {
		generate_scale_filter(self.width, self.height, self.size.as_deref(), &self.scale_mode)
	}
}

impl CanChangeFPS for AutoArgs {
	fn generate_fps_filter(&self, stream_fps: Option<f64>) -> Option<String> {
		generate_fps_filter(self.framerate, self.framerate_mult, stream_fps)
	}

	fn generate_fps_filter_explicit(&self, stream_fps: Option<f64>, target: f64) -> Option<String> {
		if let Some(fps) = stream_fps {
			let divisor = (fps / target).ceil();
			let adj_fps = (fps / divisor).round().min(target);
			return generate_fps_filter(Some(adj_fps), None, stream_fps);
		}

		None
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

impl CanCrop for GIFArgs {
	fn generate_crop_filter(&self) -> Option<String> {
		generate_crop_filter(self.crop.as_deref())
	}
}

impl CanScale for GIFArgs {
	fn generate_scale_filter(&self) -> Option<String> {
		generate_scale_filter(self.width, self.height, self.size.as_deref(), &self.scale_mode)
	}
}

impl CanChangeFPS for GIFArgs {
	fn generate_fps_filter(&self, stream_fps: Option<f64>) -> Option<String> {
		generate_fps_filter(self.framerate, self.framerate_mult, stream_fps)
	}

	fn generate_fps_filter_explicit(&self, _: Option<f64>, _: f64) -> Option<String> {
		None
	}
}

impl CanColorFilter for GIFArgs {
	fn generate_color_filters(&self) -> Option<String> {
		generate_color_sharpness_filters(self.brightness, self.contrast, self.saturation, self.sharpness)
	}
}

impl CanGeneratePalette for GIFArgs {
	fn generate_palette_filters(&self) -> Result<String> {
		generate_palette_filtergraph(
			self.palette_file.as_deref(),
			self.palette_name.as_ref(),
			self.num_colors,
			&self.stats_mode,
			self.diff_rect,
			&self.dither,
			self.bayer_scale,
		)
	}
}

impl CanSeek for QuantArgs {
	fn parse_seek(&self) -> Option<Duration> {
		parse_seek(self.seek.as_deref())
	}
}

impl CanCrop for QuantArgs {
	fn generate_crop_filter(&self) -> Option<String> {
		generate_crop_filter(self.crop.as_deref())
	}
}

impl CanScale for QuantArgs {
	fn generate_scale_filter(&self) -> Option<String> {
		generate_scale_filter(self.width, self.height, self.size.as_deref(), &self.scale_mode)
	}
}

impl CanColorFilter for QuantArgs {
	fn generate_color_filters(&self) -> Option<String> {
		generate_color_sharpness_filters(self.brightness, self.contrast, self.saturation, self.sharpness)
	}
}

impl CanGeneratePalette for QuantArgs {
	fn generate_palette_filters(&self) -> Result<String> {
		generate_palette_filtergraph(
			self.palette_file.as_deref(),
			self.palette_name.as_ref(),
			self.num_colors,
			&StatsMode::default(),
			false,
			&self.dither,
			self.bayer_scale,
		)
	}
}
