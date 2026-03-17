#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[derive(strum::Display, strum::EnumString)]
pub enum StatsMode {
	#[strum(serialize = "full")]
	#[default]
	Full,
	#[strum(serialize = "diff")]
	Diff,
	#[strum(serialize = "single")]
	Single,
}

pub struct Palettegen {
	pub max_colors: u64,
	pub reserve_transparent: bool,
	pub transparency_color: Option<String>,
	pub stats_mode: StatsMode,
}