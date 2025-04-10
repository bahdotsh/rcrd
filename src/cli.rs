use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "terminal-recorder", about = "Records terminal sessions")]
pub enum Cli {
    #[structopt(about = "Record a new terminal session")]
    Record {
        #[structopt(short, long, help = "Output file name", default_value = "demo.json")]
        output: String,
    },
    #[structopt(about = "Play back a recorded terminal session")]
    Play {
        #[structopt(help = "File to replay")]
        file: String,

        #[structopt(short, long, help = "Playback speed multiplier", default_value = "1.0")]
        speed: f32,
    },
    #[structopt(about = "Convert a recording to a GIF")]
    Export {
        #[structopt(help = "Input recording file")]
        input: String,

        #[structopt(help = "Output GIF file", default_value = "output.gif")]
        output: String,

        #[structopt(short, long, help = "Playback speed multiplier", default_value = "1.0")]
        speed: f32,

        #[structopt(short, long, help = "Terminal width", default_value = "80")]
        width: u16,

        #[structopt(short, long, help = "Terminal height", default_value = "24")]
        height: u16,

        #[structopt(short, long, help = "Font size (pixels)", default_value = "16")]
        font_size: u8,

        #[structopt(long, help = "Dark theme")]
        dark_theme: bool,
    },
}
