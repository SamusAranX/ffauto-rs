use crate::cmd_auto::ffmpeg_auto;
use crate::cmd_barcode::ffmpeg_barcode;
use crate::cmd_gif::ffmpeg_gif;
use crate::cmd_info::ffmpeg_info;
#[cfg(feature = "palette_generator")]
use crate::cmd_palettes::generate_palettes;
use crate::cmd_quant::ffmpeg_quant;
use crate::commands::{Cli, Commands};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

mod cmd_auto;
mod cmd_barcode;
mod cmd_gif;
mod cmd_info;
mod cmd_palettes;
mod cmd_quant;
mod commands;
mod commands_traits;
mod common;
mod palettes_dynamic;
mod palettes_static;
mod vec_push_ext;

fn main() -> ExitCode {
	let (cli, sub_matches) = Cli::parse_with_matches();
	let matches = sub_matches.unwrap_or_default();
	let output: &PathBuf;

	let result = match &cli.command {
		Some(Commands::Auto(args)) => {
			output = &args.output;

			let mut args = args.clone();
			args.optimize_settings();

			ffmpeg_auto(&args, &matches, cli.debug)
		}
		Some(Commands::Barcode(args)) => {
			output = &args.output;
			ffmpeg_barcode(args, cli.debug)
		}
		Some(Commands::Gif(args)) => {
			output = &args.output;
			ffmpeg_gif(args, &matches, cli.debug)
		}
		Some(Commands::Quant(args)) => {
			output = &args.output;
			ffmpeg_quant(args, &matches, cli.debug)
		}
		Some(Commands::Info(args)) => {
			return match ffmpeg_info(args) {
				Ok(()) => ExitCode::SUCCESS,
				Err(e) => {
					eprintln!("execution failed: {e}");
					ExitCode::FAILURE
				}
			};
		}
		#[cfg(feature = "palette_generator")]
		Some(Commands::Palettes(args)) => {
			return match generate_palettes(args) {
				Ok(()) => ExitCode::SUCCESS,
				Err(e) => {
					eprintln!("execution failed: {e}");
					ExitCode::FAILURE
				}
			};
		}
		None => {
			return ExitCode::FAILURE;
		}
	};

	match result {
		Ok(()) => {
			match fs::metadata(output) {
				Ok(m) => {
					#[cfg(target_os = "macos")]
					let size = humansize::format_size(m.len(), humansize::DECIMAL);
					#[cfg(not(target_os = "macos"))]
					let size = humansize::format_size(m.len(), humansize::WINDOWS);
					println!("Output file size: {size}");
				}
				Err(err) => {
					eprintln!("Can't determine output file size: {err}");
				}
			}
			ExitCode::SUCCESS
		}
		Err(e) => {
			eprintln!("execution failed: {e}");
			ExitCode::FAILURE
		}
	}
}
