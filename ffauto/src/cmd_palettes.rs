#![cfg(feature = "palette_generator")]

use crate::commands::PalettesArgs;
use crate::palettes_static::StaticPalette;
use anyhow::Result;
use clap::ValueEnum;
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::image::ImageFormat;
use imageproc::rect::Rect;
use crate::palettes_dynamic::DynamicPalette;

const IMAGE_WIDTH: u32 = 1280;
const IMAGE_HEIGHT: u32 = 64;

pub(crate) fn generate_palettes(args: &PalettesArgs) -> Result<()> {
	eprintln!("Output palettes to: {}", args.output.display());

	eprintln!("# List of static palettes");

	for static_pal in StaticPalette::value_variants() {
		let mut image = imageproc::image::RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);
		let pal = static_pal.to_palette();

		let rect_width = (IMAGE_WIDTH as f64 / pal.len() as f64).ceil() as u32;

		for (i, col) in pal.colors.iter().enumerate() {
			let rect = Rect::at((i * rect_width as usize) as i32, 0).of_size(rect_width, IMAGE_HEIGHT);
			let col = imageproc::image::Rgb { 0: [col.color.r, col.color.g, col.color.b] };
			draw_filled_rect_mut(&mut image, rect, col);
		}

		let png_name = format!("static_{static_pal}.png");
		let png_path = args.output.join(&png_name);
		image.save_with_format(&png_path, ImageFormat::Png)?;

		eprintln!("### {static_pal}");
		eprintln!(
			"![A visualization of the static \"{static_pal}\" palette]({})",
			png_path.display()
		);
		eprintln!();
	}

	eprintln!("# List of dynamic palettes");

	for dynamic_pal in DynamicPalette::value_variants() {
		let grad = dynamic_pal.to_gradient();

		let image = imageproc::image::ImageBuffer::from_fn(IMAGE_WIDTH, IMAGE_HEIGHT, |x, _| {
			imageproc::image::Rgba(grad.at(x as f32 / IMAGE_WIDTH as f32).to_rgba8())
		});

		let png_name = format!("dynamic_{dynamic_pal}.png");
		let png_path = args.output.join(&png_name);
		image.save_with_format(&png_path, ImageFormat::Png)?;

		eprintln!("### {dynamic_pal}");
		eprintln!(
			"![A visualization of the dynamic \"{dynamic_pal}\" palette]({})",
			png_path.display()
		);
		eprintln!();
	}

	Ok(())
}
