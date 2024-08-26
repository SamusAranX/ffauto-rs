use std::process::ExitCode;
use clap::Parser;

use crate::args::ProgramArgs;
use crate::ffmpeg::ffmpeg;

mod ffmpeg;
mod args;
mod vec_push_ext;

fn main() -> ExitCode {
    let args = ProgramArgs::parse();
    println!("{:?}", &args);

    match ffmpeg(&args) {
        Ok(_) => { ExitCode::SUCCESS }
        Err(e) => {
            eprintln!("fuck: {e}");
            ExitCode::FAILURE
        }
    }
}
