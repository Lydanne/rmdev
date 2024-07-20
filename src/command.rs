use clap::Parser;
pub mod clear;

#[derive(Parser)]
#[clap(
    name = "Rmdev",
    version = "0.0.1",
    author = "@wumacoder",
    about = "a clear dev junk file."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Parser)]
pub enum Commands {
    /// clear junk file
    Clear(clear::Clear),
    // /// start a tui.
    // UI(RunUI),
}

// #[derive(clap::Parser, Debug)]
// pub struct RunUI {
//     /// eg: qxg
//     pub target: String,

//     /// regex filter oplog
//     #[clap(short, long)]
//     pub filter: Option<String>,
// }
