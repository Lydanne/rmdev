use clap::Parser;
use command::Cli;
use tokio::runtime::Builder;

mod command;
mod scan_category;
mod signal;
mod ui;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.commands {
        command::Commands::Clear(args) => {
            args.run().await?;
        }
    }

    Ok(())
}

fn main() {
    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let r = run().await;
        if let Err(err) = r {
            eprintln!("Error: {}", err);
        }
    });
}
