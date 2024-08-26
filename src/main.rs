use clap::Parser;

use crate::args::ProgramArgs;

mod ffmpeg;
mod args;

fn main() {
    let args = ProgramArgs::parse();
    println!("{:?}", &args);

    // ffmpeg(wat);
}
