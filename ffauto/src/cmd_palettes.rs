#![cfg(feature = "palette_generator")]

use crate::commands::PalettesArgs;
use crate::palettes::{get_builtin_palette, BuiltInPalette};
use anyhow::Result;
use clap::ValueEnum;
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::image::ImageFormat;
use imageproc::rect::Rect;

const IMAGE_WIDTH: u32 = 512;
const IMAGE_HEIGHT: u32 = 32;

pub(crate) fn generate_palettes(args: &PalettesArgs) -> Result<()> {
	eprintln!("Output palettes to: {}", args.output.display());

	for built_in_pal in BuiltInPalette::value_variants() {
		let mut image = imageproc::image::RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);
		let pal = get_builtin_palette(built_in_pal);

		let rect_width = (IMAGE_WIDTH as f64 / pal.len() as f64).ceil() as u32;

		for (i, col) in pal.colors.iter().enumerate() {
			let rect = Rect::at((i * rect_width as usize) as i32, 0).of_size(rect_width, IMAGE_HEIGHT);
			let col = imageproc::image::Rgb { 0: [col.color.r, col.color.g, col.color.b] };
			draw_filled_rect_mut(&mut image, rect, col);
		}

		let png_name = format!("{built_in_pal}.png");
		let png_path = args.output.join(&png_name);
		image.save_with_format(&png_path, ImageFormat::Png)?;

		eprintln!("### {built_in_pal}");
		eprintln!(
			"![A visualization of the \"{built_in_pal}\" palette]({})",
			png_path.display()
		);
		eprintln!();
	}

	Ok(())
}
