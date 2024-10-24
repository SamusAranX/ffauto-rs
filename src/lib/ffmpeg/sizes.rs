use crate::ffmpeg::enums::Size;
use anyhow::Result;
use regex::{Captures, Regex};

pub fn parse_ffmpeg_size<S: Into<String>>(size: S) -> Result<Size> {
	let size: String = size.into();

	// https://github.com/FFmpeg/FFmpeg/blob/00f5a34c9a5f0adee28aca11971918d6aca48745/libavutil/parseutils.c#L76
	match size.as_str() {
		"ntsc" => Ok(Size::new(720, 480)),
		"pal" => Ok(Size::new(720, 576)),
		"qntsc" => Ok(Size::new(352, 240)),
		"qpal" => Ok(Size::new(352, 288)),
		"sntsc" => Ok(Size::new(640, 480)),
		"spal" => Ok(Size::new(768, 576)),
		"film" => Ok(Size::new(352, 240)),
		"ntsc-film" => Ok(Size::new(352, 240)),
		"sqcif" => Ok(Size::new(128, 96)),
		"qcif" => Ok(Size::new(176, 144)),
		"cif" => Ok(Size::new(352, 288)),
		"4cif" => Ok(Size::new(704, 576)),
		"16cif" => Ok(Size::new(1408, 1152)),
		"qqvga" => Ok(Size::new(160, 120)),
		"qvga" => Ok(Size::new(320, 240)),
		"vga" => Ok(Size::new(640, 480)),
		"svga" => Ok(Size::new(800, 600)),
		"xga" => Ok(Size::new(1024, 768)),
		"uxga" => Ok(Size::new(1600, 1200)),
		"qxga" => Ok(Size::new(2048, 1536)),
		"sxga" => Ok(Size::new(1280, 1024)),
		"qsxga" => Ok(Size::new(2560, 2048)),
		"hsxga" => Ok(Size::new(5120, 4096)),
		"wvga" => Ok(Size::new(852, 480)),
		"wxga" => Ok(Size::new(1366, 768)),
		"wsxga" => Ok(Size::new(1600, 1024)),
		"wuxga" => Ok(Size::new(1920, 1200)),
		"woxga" => Ok(Size::new(2560, 1600)),
		"wqhd" => Ok(Size::new(2560, 1440)),
		"wqsxga" => Ok(Size::new(3200, 2048)),
		"wquxga" => Ok(Size::new(3840, 2400)),
		"whsxga" => Ok(Size::new(6400, 4096)),
		"whuxga" => Ok(Size::new(7680, 4800)),
		"cga" => Ok(Size::new(320, 200)),
		"ega" => Ok(Size::new(640, 350)),
		"hd480" => Ok(Size::new(852, 480)),
		"hd720" => Ok(Size::new(1280, 720)),
		"hd1080" => Ok(Size::new(1920, 1080)),
		"quadhd" => Ok(Size::new(2560, 1440)),
		"2k" => Ok(Size::new(2048, 1080)),
		"2kdci" => Ok(Size::new(2048, 1080)),
		"2kflat" => Ok(Size::new(1998, 1080)),
		"2kscope" => Ok(Size::new(2048, 858)),
		"4k" => Ok(Size::new(4096, 2160)),
		"4kdci" => Ok(Size::new(4096, 2160)),
		"4kflat" => Ok(Size::new(3996, 2160)),
		"4kscope" => Ok(Size::new(4096, 1716)),
		"nhd" => Ok(Size::new(640, 360)),
		"hqvga" => Ok(Size::new(240, 160)),
		"wqvga" => Ok(Size::new(400, 240)),
		"fwqvga" => Ok(Size::new(432, 240)),
		"hvga" => Ok(Size::new(480, 320)),
		"qhd" => Ok(Size::new(960, 540)),
		"uhd2160" => Ok(Size::new(3840, 2160)),
		"uhd4320" => Ok(Size::new(7680, 4320)),

		_ => {
			let re = Regex::new(r"^(?P<W>\d+)x(?P<H>\d+)$").unwrap();
			let groups: Captures = match re.captures(&size) {
				None => { anyhow::bail!("Invalid size string \"{size}\" provided") }
				Some(captures) => captures
			};

			if let (Some(w), Some(h)) = (groups.name("W"), groups.name("H")) {
				let width = w.as_str().parse::<u64>().unwrap_or_default();
				let height = h.as_str().parse::<u64>().unwrap_or_default();
				return Ok(Size::new(width, height));
			}

			anyhow::bail!("Couldn't parse size string \"{size}\"")
		}
	}
}