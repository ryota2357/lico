use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run
    Run { file: std::path::PathBuf },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file } => run::start(file),
    }
}
