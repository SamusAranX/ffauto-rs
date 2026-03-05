use serde::de;

pub mod enums;
pub mod enums_impl;
#[allow(clippy::module_inception)]
pub mod ffmpeg;
pub mod ffprobe;
pub mod ffprobe_struct;
mod filters;
pub mod size;
pub mod timestamps;

fn deserialize_bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
	D: de::Deserializer<'de>,
{
	let s: u8 = de::Deserialize::deserialize(deserializer)?;

	match s {
		0 => Ok(false),
		_ => Ok(true),
	}
}
