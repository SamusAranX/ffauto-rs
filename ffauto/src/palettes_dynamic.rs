use colorgrad::Gradient;
use ffmpeg::palettes::palette::{Color, Palette};
// from colorgrad-rs
// https://github.com/mazznoer/colorgrad-rs/blob/master/PRESET.md

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, strum::Display)]
pub enum DynamicPalette {
	// Diverging
	BrBg,
	PrGn,
	PiYg,
	PuOr,
	RdBu,
	RdGy,
	RdYlBu,
	RdYlGn,
	Spectral,

	// Sequential (Single Hue)
	Blues,
	Greens,
	Greys,
	Oranges,
	Purples,
	Reds,

	// Sequential (Multi-Hue)
	Turbo,
	Viridis,
	Inferno,
	Magma,
	Plasma,
	Cividis,
	Warm,
	Cool,
	Cubehelix,
	BuGn,
	BuPu,
	GnBu,
	OrRd,
	PuBuGn,
	PuBu,
	PuRd,
	RdPu,
	YlGnBu,
	YlGn,
	YlOrBr,
	YlOrRd,

	// Cyclical
	Rainbow,
	Sinebow,
}

impl DynamicPalette {
	pub fn to_gradient(&self) -> Box<dyn Gradient> {
		match self {
			// Diverging
			DynamicPalette::BrBg => colorgrad::preset::br_bg().boxed(),
			DynamicPalette::PrGn => colorgrad::preset::pr_gn().boxed(),
			DynamicPalette::PiYg => colorgrad::preset::pi_yg().boxed(),
			DynamicPalette::PuOr => colorgrad::preset::pu_or().boxed(),
			DynamicPalette::RdBu => colorgrad::preset::rd_bu().boxed(),
			DynamicPalette::RdGy => colorgrad::preset::rd_gy().boxed(),
			DynamicPalette::RdYlBu => colorgrad::preset::rd_yl_bu().boxed(),
			DynamicPalette::RdYlGn => colorgrad::preset::rd_yl_gn().boxed(),
			DynamicPalette::Spectral => colorgrad::preset::spectral().boxed(),

			// Sequential (Single Hue)
			DynamicPalette::Blues => colorgrad::preset::blues().boxed(),
			DynamicPalette::Greens => colorgrad::preset::greens().boxed(),
			DynamicPalette::Greys => colorgrad::preset::greys().boxed(),
			DynamicPalette::Oranges => colorgrad::preset::oranges().boxed(),
			DynamicPalette::Purples => colorgrad::preset::purples().boxed(),
			DynamicPalette::Reds => colorgrad::preset::reds().boxed(),

			// Sequential (Multi-Hue)
			DynamicPalette::Turbo => colorgrad::preset::turbo().boxed(),
			DynamicPalette::Viridis => colorgrad::preset::viridis().boxed(),
			DynamicPalette::Inferno => colorgrad::preset::inferno().boxed(),
			DynamicPalette::Magma => colorgrad::preset::magma().boxed(),
			DynamicPalette::Plasma => colorgrad::preset::plasma().boxed(),
			DynamicPalette::Cividis => colorgrad::preset::cividis().boxed(),
			DynamicPalette::Warm => colorgrad::preset::warm().boxed(),
			DynamicPalette::Cool => colorgrad::preset::cool().boxed(),
			DynamicPalette::Cubehelix => colorgrad::preset::cubehelix_default().boxed(),
			DynamicPalette::BuGn => colorgrad::preset::bu_gn().boxed(),
			DynamicPalette::BuPu => colorgrad::preset::bu_pu().boxed(),
			DynamicPalette::GnBu => colorgrad::preset::gn_bu().boxed(),
			DynamicPalette::OrRd => colorgrad::preset::or_rd().boxed(),
			DynamicPalette::PuBuGn => colorgrad::preset::pu_bu_gn().boxed(),
			DynamicPalette::PuBu => colorgrad::preset::pu_bu().boxed(),
			DynamicPalette::PuRd => colorgrad::preset::pu_rd().boxed(),
			DynamicPalette::RdPu => colorgrad::preset::rd_pu().boxed(),
			DynamicPalette::YlGnBu => colorgrad::preset::yl_gn_bu().boxed(),
			DynamicPalette::YlGn => colorgrad::preset::yl_gn().boxed(),
			DynamicPalette::YlOrBr => colorgrad::preset::yl_or_br().boxed(),
			DynamicPalette::YlOrRd => colorgrad::preset::yl_or_rd().boxed(),

			// Cyclical
			DynamicPalette::Rainbow => colorgrad::preset::rainbow().boxed(),
			DynamicPalette::Sinebow => colorgrad::preset::sinebow().boxed(),
		}
	}

	pub fn to_palette(&self, num_colors: u16) -> Palette {
		let gradient = self.to_gradient();

		let palette_colors = gradient
			.colors(num_colors as usize)
			.iter()
			.map(|c| Color::from_f32(c.r, c.g, c.b))
			.collect::<Vec<_>>();
		Palette::from(palette_colors)
	}
}
