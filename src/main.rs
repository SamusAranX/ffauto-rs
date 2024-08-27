use crate::cmd_auto::ffmpeg_auto;
use crate::cmd_gif::ffmpeg_gif;
use crate::cmd_quant::ffmpeg_quant;
use crate::commands::{Cli, Commands};
use clap::Parser;
use std::process::ExitCode;

mod cmd_auto;
mod cmd_gif;
mod cmd_quant;
mod commands;
mod vec_push_ext;
mod cmd;

fn main() -> ExitCode {
	let cli = Cli::parse();

	let result = match &cli.command {
		Some(Commands::Auto(args)) => {
			ffmpeg_auto(&cli, args)
		}
		Some(Commands::Gif(args)) => {
			ffmpeg_gif(&cli, args)
		}
		Some(Commands::Quant(args)) => {
			ffmpeg_quant(&cli, args)
		}
		_ => { Ok(()) }
	};

	match result {
		Ok(_) => { ExitCode::SUCCESS }
		Err(e) => {
			eprintln!("execution failed: {e}");
			ExitCode::FAILURE
		}
	}
}
