mod cmd;
mod config;
mod git;
mod hook;
mod integration;
mod repo_source;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "projj",
    version,
    about = "Manage git repositories with directory conventions"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize configuration
    Init,
    /// Add repository
    Add {
        /// Repository URL, short form (owner/repo), or local path
        repo: String,
    },
    /// Find repository
    Find {
        /// Keyword to search (optional, opens fzf if omitted)
        keyword: Option<String>,
    },
    /// Remove repository
    Remove {
        /// Keyword to match repository
        keyword: String,
    },
    /// Run script in repository
    Run {
        /// Script name or shell command
        script: String,
        /// Run in all repositories
        #[arg(long)]
        all: bool,
        /// Regex to filter repositories (with --all)
        #[arg(long, value_name = "PATTERN")]
        r#match: Option<String>,
    },
    /// List all repositories
    List {
        /// Force rebuild index
        #[arg(long)]
        rebuild: bool,
    },
    /// Generate shell completions
    #[command(hide = true)]
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd::init::run()?,
        Commands::Add { repo } => cmd::add::run(&repo)?,
        Commands::Find { keyword } => cmd::find::run(keyword.as_deref())?,
        Commands::Remove { keyword } => cmd::remove::run(&keyword)?,
        Commands::Run {
            script,
            all,
            r#match,
        } => cmd::run::run(&script, all, r#match.as_deref())?,
        Commands::List { rebuild: _ } => cmd::list::run()?,
        Commands::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "projj", &mut std::io::stdout());
        }
    }

    Ok(())
}
