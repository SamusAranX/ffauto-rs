use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use humansize::DECIMAL;

use crate::cmd_auto::ffmpeg_auto;
use crate::cmd_gif::ffmpeg_gif;
use crate::cmd_info::ffmpeg_info;
use crate::cmd_quant::ffmpeg_quant;
use crate::commands::{Cli, Commands};

mod cmd_auto;
mod cmd_gif;
mod cmd_info;
mod cmd_quant;
mod commands;
mod common;
mod palettes;
mod vec_push_ext;

fn main() -> ExitCode {
	let cli = Cli::parse();
	let output: &PathBuf;

	// println!("{cli:?}");

	let result = match &cli.command {
		Some(Commands::Auto(args)) => {
			output = &args.output;

			let (cli, args) = {
				let mut optimized_cli = cli.clone();
				optimized_cli.optimize_settings(&args.optimize_target);

				let mut optimized_args = args.clone();
				optimized_args.optimize_settings();

				(optimized_cli, optimized_args)
			};

			ffmpeg_auto(&cli, &args)
		}
		Some(Commands::Gif(args)) => {
			output = &args.output;
			ffmpeg_gif(&cli, args)
		}
		Some(Commands::Quant(args)) => {
			output = &args.output;
			ffmpeg_quant(&cli, args)
		}
		Some(Commands::Info(args)) => {
			return match ffmpeg_info(args) {
				Ok(_) => {
					ExitCode::SUCCESS
				}
				Err(e) => {
					eprintln!("execution failed: {e}");
					ExitCode::FAILURE
				}
			}
		}
		None => {
			return ExitCode::FAILURE;
		}
	};

	match result {
		Ok(_) => {
			match fs::metadata(output) {
				Ok(m) => {
					let size = humansize::format_size(m.len(), DECIMAL);
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
