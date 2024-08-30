use ffauto_rs::palettes::palette::Palette;

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum BuiltInPalette {
	Cmyk, // actually CMYK plus white
	Windows,
	Macintosh,
	WebSafe,
	Grayscale,
	Monochrome,

	// by Adigun Polack
	// https://twitter.com/AdigunPolack
	AAP64, // https://pixeljoint.com/pixelart/119466.htm
	AAPMicro12, // https://pixeljoint.com/pixelart/121151.htm
	AAPRadiantXV,
	AAPSplendor128, // https://pixeljoint.com/pixelart/120714.htm
	SimpleJPC16, // https://pixeljoint.com/pixelart/119844.htm

	// by Arne Niklas Jansson
	// https://androidarts.com/palette/16pal.htm
	A64,
	ARNE16,
	ARNE32,
	CgArne,
	CopperTech,
	CpcBoy,
	ErogeCopper,
	Jmp,
	Psygnosia,

	// by Davit Masia
	// https://twitter.com/DavitMasia
	Matriax8c,

	// by Richard "DawnBringer" Fhager
	// https://hem.fyristorg.com/dawnbringer/
	DB8, // https://pixeljoint.com/forum/forum_posts.asp?TID=26050
	DB16, // https://pixeljoint.com/forum/forum_posts.asp?TID=12795
	DB32, // https://pixeljoint.com/forum/forum_posts.asp?TID=16247

	// by ENDESGA Studios
	// https://twitter.com/ENDESGA
	ARQ4,
	ARQ16,
	EDG8,
	EDG16,
	EDG32,
	EN4,
	ENOS16,
	HEPT32,

	// Hardware Palettes
	// https://github.com/aseprite/aseprite/tree/8323a555007e1db9670b098ce4b1b9c5f8b3d7ad/data/extensions/hardware-palettes
	AppleII,
	Atari2600Ntsc,
	Atari2600Pal,
	Cga,
	Cga0,
	Cga0High,
	Cga1,
	Cga1High,
	Cga3rd,
	Cga3rdHigh,
	CommodorePlus4,
	CommodoreVic20,
	Commodore64,
	Cpc,
	GameBoy,
	GameBoyColor,
	MasterSystem,
	MSX1,
	MSX2,
	Nes,
	NesNtsc,
	Teletext,
	VGA13h,
	VirtualBoy,
	ZXSpectrum,

	// by Hyohnoo
	// https://twitter.com/Hyohnoo
	Mail24,

	// by Javier Guerrero
	// https://twitter.com/Xavier_Gd
	Nyx8,

	// by Joseph White
	// https://www.pico-8.com/
	Pico8,

	// by PineTreePizza
	// https://twitter.com/PineTreePizza
	Bubblegum16,
	Rosy42,

	// Software Palettes
	// https://github.com/aseprite/aseprite/tree/8323a555007e1db9670b098ce4b1b9c5f8b3d7ad/data/extensions/software-palettes
	GoogleUI,
	Minecraft,
	Monokai,
	SmileBasic,
	Solarized,
	Win16,
	X11,

	// by Zughy
	// https://twitter.com/_Zughy
	Zughy32
}

pub fn get_builtin_palette(pal: &BuiltInPalette) -> Palette {
	match pal {
		BuiltInPalette::Cmyk => Palette::from(vec![0x0, 0xffff00, 0x00ffff, 0xff00ff, 0xffffff]),
		BuiltInPalette::Windows => Palette::from_hex_string(include_str!("palettes/windows.hex")).unwrap(),
		BuiltInPalette::Macintosh => Palette::from_hex_string(include_str!("palettes/macintosh.hex")).unwrap(),
		BuiltInPalette::WebSafe => Palette::from_hex_string(include_str!("palettes/websafe.hex")).unwrap(),
		BuiltInPalette::Grayscale => Palette::from_hex_string(include_str!("palettes/grayscale.hex")).unwrap(),
		BuiltInPalette::Monochrome => Palette::from(vec![0x0, 0xffffff]),

		BuiltInPalette::AAP64 => Palette::from_gpl_string(include_str!("palettes/adigunpolack-palettes/aap-64.gpl")).unwrap(),
		BuiltInPalette::AAPMicro12 => Palette::from_gpl_string(include_str!("palettes/adigunpolack-palettes/aap-micro12.gpl")).unwrap(),
		BuiltInPalette::AAPRadiantXV => Palette::from_gpl_string(include_str!("palettes/adigunpolack-palettes/aap-radiantxv.gpl")).unwrap(),
		BuiltInPalette::AAPSplendor128 => Palette::from_gpl_string(include_str!("palettes/adigunpolack-palettes/aap-splendor128.gpl")).unwrap(),
		BuiltInPalette::SimpleJPC16 => Palette::from_gpl_string(include_str!("palettes/adigunpolack-palettes/simplejpc-16.gpl")).unwrap(),

		BuiltInPalette::A64 => Palette::from_gpl_string(include_str!("palettes/arne-palettes/a64.gpl")).unwrap(),
		BuiltInPalette::ARNE16 => Palette::from_gpl_string(include_str!("palettes/arne-palettes/arne16.gpl")).unwrap(),
		BuiltInPalette::ARNE32 => Palette::from_gpl_string(include_str!("palettes/arne-palettes/arne32.gpl")).unwrap(),
		BuiltInPalette::CgArne => Palette::from_gpl_string(include_str!("palettes/arne-palettes/cg-arne.gpl")).unwrap(),
		BuiltInPalette::CopperTech => Palette::from_gpl_string(include_str!("palettes/arne-palettes/copper-tech.gpl")).unwrap(),
		BuiltInPalette::CpcBoy => Palette::from_gpl_string(include_str!("palettes/arne-palettes/cpc-boy.gpl")).unwrap(),
		BuiltInPalette::ErogeCopper => Palette::from_gpl_string(include_str!("palettes/arne-palettes/eroge-copper.gpl")).unwrap(),
		BuiltInPalette::Jmp => Palette::from_gpl_string(include_str!("palettes/arne-palettes/jmp.gpl")).unwrap(),
		BuiltInPalette::Psygnosia => Palette::from_gpl_string(include_str!("palettes/arne-palettes/psygnosia.gpl")).unwrap(),

		BuiltInPalette::Matriax8c => Palette::from_gpl_string(include_str!("palettes/davitmasia-palettes/matriax8c.gpl")).unwrap(),

		BuiltInPalette::DB8 => Palette::from_gpl_string(include_str!("palettes/dawnbringer-palettes/db8.gpl")).unwrap(),
		BuiltInPalette::DB16 => Palette::from_gpl_string(include_str!("palettes/dawnbringer-palettes/db16.gpl")).unwrap(),
		BuiltInPalette::DB32 => Palette::from_gpl_string(include_str!("palettes/dawnbringer-palettes/db32.gpl")).unwrap(),

		BuiltInPalette::ARQ4 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/arq4.gpl")).unwrap(),
		BuiltInPalette::ARQ16 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/arq16.gpl")).unwrap(),
		BuiltInPalette::EDG8 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/edg8.gpl")).unwrap(),
		BuiltInPalette::EDG16 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/edg16.gpl")).unwrap(),
		BuiltInPalette::EDG32 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/edg32.gpl")).unwrap(),
		BuiltInPalette::EN4 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/en4.gpl")).unwrap(),
		BuiltInPalette::ENOS16 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/enos16.gpl")).unwrap(),
		BuiltInPalette::HEPT32 => Palette::from_gpl_string(include_str!("palettes/endesga-palettes/hept32.gpl")).unwrap(),

		BuiltInPalette::AppleII => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/apple-ii.gpl")).unwrap(),
		BuiltInPalette::Atari2600Ntsc => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/atari2600-ntsc.gpl")).unwrap(),
		BuiltInPalette::Atari2600Pal => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/atari2600-pal.gpl")).unwrap(),
		BuiltInPalette::Cga => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cga.gpl")).unwrap(),
		BuiltInPalette::Cga0 => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cga0.gpl")).unwrap(),
		BuiltInPalette::Cga0High => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cga0hi.gpl")).unwrap(),
		BuiltInPalette::Cga1 => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cga1.gpl")).unwrap(),
		BuiltInPalette::Cga1High => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cga1hi.gpl")).unwrap(),
		BuiltInPalette::Cga3rd => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cga3rd.gpl")).unwrap(),
		BuiltInPalette::Cga3rdHigh => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cga3rdhi.gpl")).unwrap(),
		BuiltInPalette::CommodorePlus4 => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/commodore-plus4.gpl")).unwrap(),
		BuiltInPalette::CommodoreVic20 => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/commodore-vic20.gpl")).unwrap(),
		BuiltInPalette::Commodore64 => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/commodore64.gpl")).unwrap(),
		BuiltInPalette::Cpc => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/cpc.gpl")).unwrap(),
		BuiltInPalette::GameBoy => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/gameboy.gpl")).unwrap(),
		BuiltInPalette::GameBoyColor => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/gameboy-color-type1.gpl")).unwrap(),
		BuiltInPalette::MasterSystem => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/master-system.gpl")).unwrap(),
		BuiltInPalette::MSX1 => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/msx1.gpl")).unwrap(),
		BuiltInPalette::MSX2 => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/msx2.gpl")).unwrap(),
		BuiltInPalette::Nes => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/nes.gpl")).unwrap(),
		BuiltInPalette::NesNtsc => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/nes-ntsc.gpl")).unwrap(),
		BuiltInPalette::Teletext => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/teletext.gpl")).unwrap(),
		BuiltInPalette::VGA13h => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/vga-13h.gpl")).unwrap(),
		BuiltInPalette::VirtualBoy => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/virtualboy.gpl")).unwrap(),
		BuiltInPalette::ZXSpectrum => Palette::from_gpl_string(include_str!("palettes/hardware-palettes/zx-spectrum.gpl")).unwrap(),

		BuiltInPalette::Mail24 => Palette::from_gpl_string(include_str!("palettes/hyohnoo-palettes/mail24.gpl")).unwrap(),

		BuiltInPalette::Nyx8 => Palette::from_gpl_string(include_str!("palettes/javierguerrero-palettes/nyx8.gpl")).unwrap(),

		BuiltInPalette::Pico8 => Palette::from_gpl_string(include_str!("palettes/pico8-palette/pico-8.gpl")).unwrap(),

		BuiltInPalette::Bubblegum16 => Palette::from_gpl_string(include_str!("palettes/pinetreepizza-palettes/bubblegum-16.gpl")).unwrap(),
		BuiltInPalette::Rosy42 => Palette::from_gpl_string(include_str!("palettes/pinetreepizza-palettes/rosy-42.gpl")).unwrap(),

		BuiltInPalette::GoogleUI => Palette::from_gpl_string(include_str!("palettes/software-palettes/google-ui.gpl")).unwrap(),
		BuiltInPalette::Minecraft => Palette::from_gpl_string(include_str!("palettes/software-palettes/minecraft.gpl")).unwrap(),
		BuiltInPalette::Monokai => Palette::from_gpl_string(include_str!("palettes/software-palettes/monokai.gpl")).unwrap(),
		BuiltInPalette::SmileBasic => Palette::from_gpl_string(include_str!("palettes/software-palettes/smile-basic.gpl")).unwrap(),
		BuiltInPalette::Solarized => Palette::from_gpl_string(include_str!("palettes/software-palettes/solarized.gpl")).unwrap(),
		BuiltInPalette::Win16 => Palette::from_gpl_string(include_str!("palettes/software-palettes/win16.gpl")).unwrap(),
		BuiltInPalette::X11 => Palette::from_gpl_string(include_str!("palettes/software-palettes/x11.gpl")).unwrap(),

		BuiltInPalette::Zughy32 => Palette::from_gpl_string(include_str!("palettes/zughy-palettes/zughy-32.gpl")).unwrap(),
	}
}