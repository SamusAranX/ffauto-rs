use crate::cmd_auto::ffmpeg_auto;
use crate::cmd_gif::ffmpeg_gif;
use crate::cmd_quant::ffmpeg_quant;
use crate::commands::{Cli, Commands};
use clap::Parser;
use humansize::DECIMAL;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

mod cmd_auto;
mod cmd_gif;
mod cmd_quant;
mod commands;
mod common;
mod palettes;
mod vec_push_ext;

fn main() -> ExitCode {
	let cli = Cli::parse();
	let output: &PathBuf;

	let result = match &cli.command {
		Some(Commands::Auto(args)) => {
			output = &args.output;
			ffmpeg_auto(&cli, args)
		}
		Some(Commands::Gif(args)) => {
			output = &args.output;
			ffmpeg_gif(&cli, args)
		}
		Some(Commands::Quant(args)) => {
			output = &args.output;
			ffmpeg_quant(&cli, args)
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
