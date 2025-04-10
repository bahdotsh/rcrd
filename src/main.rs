mod cli;
mod export;
mod recording;
mod terminal;
mod utils;

use cli::Cli;
use recording::{playback, recorder};
use std::io;
use structopt::StructOpt;

fn main() -> io::Result<()> {
    let opt = Cli::from_args();

    match opt {
        Cli::Record { output } => recorder::record_session(&output)?,
        Cli::Play { file, speed } => playback::play_session(&file, speed)?,
        Cli::Export {
            input,
            output,
            speed,
            width,
            height,
            font_size,
            dark_theme,
        } => export::gif::export_to_gif(
            &input, &output, speed, width, height, font_size, dark_theme,
        )?,
    }

    Ok(())
}
