use crate::commands::PalettesArgs;
use crate::palettes::{get_builtin_palette, BuiltInPalette};
use anyhow::Result;
use clap::ValueEnum;

pub(crate) fn generate_palettes(args: &PalettesArgs) -> Result<()> {
	eprintln!("Output palettes to: {}", args.output.display());

	for built_in_pal in BuiltInPalette::value_variants() {
		let pal = get_builtin_palette(built_in_pal);
		eprintln!("{built_in_pal}: {} colors", pal.len());
	}

	Ok(())
}
