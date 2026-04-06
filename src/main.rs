mod cmd;
mod color;
mod config;
mod git;
mod hook;
mod repo_source;
mod select;
mod task;

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
        /// Extra arguments passed to the script (after --)
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// List all repositories
    List {
        /// Output raw paths only (for piping)
        #[arg(long)]
        raw: bool,
    },
    /// Output shell setup (completions + `p()` function)
    #[command(name = "shell-setup", hide = true)]
    ShellSetup {
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
            args,
        } => cmd::run::run(&script, all, r#match.as_deref(), &args)?,
        Commands::List { raw } => cmd::list::run(raw)?,
        Commands::ShellSetup { shell } => {
            // Output completions
            clap_complete::generate(shell, &mut Cli::command(), "projj", &mut std::io::stdout());

            // Output p() function
            match shell {
                Shell::Zsh => {
                    println!();
                    println!("# projj: quick navigation");
                    println!("p() {{");
                    println!("  local dir");
                    println!("  dir=$(projj find \"$@\")");
                    println!("  [ -n \"$dir\" ] && cd \"$dir\"");
                    println!("}}");
                    println!();
                    println!("# projj run: complete task names (first arg only)");
                    println!("_projj_run_tasks() {{");
                    println!("  [[ $CURRENT -ne 2 ]] && return");
                    println!("  local tasks=()");
                    println!("  local config=\"$HOME/.projj/config.toml\"");
                    println!("  if [[ -f \"$config\" ]]; then");
                    println!(
                        "    tasks+=($(sed -n '/^\\[tasks\\]/,/^\\[/{{ /^[a-zA-Z_-]* *=/s/ *=.*//p }}' \"$config\" 2>/dev/null))"
                    );
                    println!("  fi");
                    println!("  local tasks_dir=\"$HOME/.projj/tasks\"");
                    println!("  if [[ -d \"$tasks_dir\" ]]; then");
                    println!("    tasks+=($(ls \"$tasks_dir\" 2>/dev/null))");
                    println!("  fi");
                    println!("  compadd -a tasks");
                    println!("}}");
                    println!("compdef '_projj_run_tasks' 'projj run'");
                }
                Shell::Bash => {
                    println!();
                    println!("# projj: quick navigation");
                    println!("p() {{");
                    println!("  local dir");
                    println!("  dir=$(projj find \"$@\")");
                    println!("  [ -n \"$dir\" ] && cd \"$dir\"");
                    println!("}}");
                    println!();
                    println!("# projj run: complete task names (first arg only)");
                    println!("_projj_run_complete() {{");
                    println!("  [[ $COMP_CWORD -ne 2 ]] && return");
                    println!("  local tasks=\"\"");
                    println!("  local config=\"$HOME/.projj/config.toml\"");
                    println!("  if [[ -f \"$config\" ]]; then");
                    println!(
                        "    tasks+=\"$(sed -n '/^\\[tasks\\]/,/^\\[/{{ /^[a-zA-Z_-]* *=/s/ *=.*//p }}' \"$config\" 2>/dev/null)\""
                    );
                    println!("  fi");
                    println!("  local tasks_dir=\"$HOME/.projj/tasks\"");
                    println!("  if [[ -d \"$tasks_dir\" ]]; then");
                    println!("    tasks+=\" $(ls \"$tasks_dir\" 2>/dev/null)\"");
                    println!("  fi");
                    println!(
                        "  COMPREPLY=($(compgen -W \"$tasks\" -- \"${{COMP_WORDS[COMP_CWORD]}}\"))"
                    );
                    println!("}}");
                    println!("complete -F _projj_run_complete projj run");
                }
                Shell::Fish => {
                    println!();
                    println!("# projj: quick navigation");
                    println!("function p");
                    println!("  set -l dir (projj find $argv)");
                    println!("  test -n \"$dir\"; and cd $dir");
                    println!("end");
                    println!();
                    println!("# projj run: complete task names");
                    println!("complete -c projj -n '__fish_seen_subcommand_from run' -a '(begin");
                    println!("  set -l config $HOME/.projj/config.toml");
                    println!(
                        "  test -f $config; and sed -n \"/^\\\\[tasks\\\\]/,/^\\\\[/{{ /^[a-zA-Z_-]* *=/s/ *=.*//p }}\" $config 2>/dev/null"
                    );
                    println!("  set -l tasks_dir $HOME/.projj/tasks");
                    println!("  test -d $tasks_dir; and ls $tasks_dir 2>/dev/null");
                    println!("end)'");
                }
                _ => {}
            }
        }
    }

    Ok(())
}
