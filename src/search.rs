use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;

use crate::repo_source::Repo;

// ── ANSI helpers ──

pub const RESET: &str = "\x1b[0m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";

pub const GROUP_COLORS: &[&str] = &[
    "\x1b[48;5;24m\x1b[97m",  // dark blue
    "\x1b[48;5;22m\x1b[97m",  // dark green
    "\x1b[48;5;94m\x1b[97m",  // dark orange
    "\x1b[48;5;30m\x1b[97m",  // teal
    "\x1b[48;5;238m\x1b[97m", // dark gray
];

// ── Group key ──

pub fn group_key_for(repo: &Repo, has_multiple_bases: bool) -> String {
    if has_multiple_bases {
        let base_name = repo
            .base
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        format!("{}/{}", base_name, repo.host)
    } else {
        repo.host.clone()
    }
}

// ── Display builders ──

/// Build indexed display items for fzf: `INDEX\tCOLORED_LINE` per repo.
pub fn build_indexed_items(repos: &[Repo], has_multiple_bases: bool) -> Vec<String> {
    let group_colors = assign_group_colors(repos, has_multiple_bases);

    repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let group_key = group_key_for(repo, has_multiple_bases);
            let color = group_colors[&group_key];
            format!(
                "{i}\t{} {} {} {}  {}{}{}",
                color,
                group_key,
                RESET,
                repo.short_key(),
                DIM,
                repo.git_url(),
                RESET,
            )
        })
        .collect()
}

/// Print single repo info to stderr (for find with one match).
pub fn print_repo_info(repo: &Repo, has_multiple_bases: bool) {
    let group_key = group_key_for(repo, has_multiple_bases);
    let color = GROUP_COLORS[0];
    eprintln!("{color} {group_key} {RESET}");
    eprintln!(
        "    {}  {}{}{}",
        repo.short_key(),
        DIM,
        repo.git_url(),
        RESET
    );
}

/// Assign a color to each group key.
fn assign_group_colors<'a>(repos: &[Repo], has_multiple_bases: bool) -> BTreeMap<String, &'a str> {
    let mut group_colors: BTreeMap<String, &str> = BTreeMap::new();
    let mut color_idx = 0;

    for repo in repos {
        let group_key = group_key_for(repo, has_multiple_bases);
        if let std::collections::btree_map::Entry::Vacant(e) = group_colors.entry(group_key) {
            e.insert(GROUP_COLORS[color_idx % GROUP_COLORS.len()]);
            color_idx += 1;
        }
    }

    group_colors
}

// ── fzf / dialoguer ──

/// Check if a command exists in PATH.
fn has_command(name: &str) -> bool {
    let check = if cfg!(windows) { "where" } else { "which" };
    Command::new(check)
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn has_fzf() -> bool {
    has_command("fzf")
}

/// Run fzf with indexed items ("INDEX\tDISPLAY" per line).
/// fzf shows only the display part (`--with-nth=2..`) but returns the full line.
/// Parses the index from the returned line. Returns None if cancelled.
pub fn fzf_indexed(items: &[String], query: Option<&str>) -> Result<Option<usize>> {
    if items.is_empty() {
        return Ok(None);
    }

    if has_fzf() {
        let mut cmd = Command::new("fzf");
        cmd.args(["--ansi", "--delimiter=\t", "--with-nth=2.."])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        if let Some(q) = query {
            cmd.args(["--query", q]);
        }

        let mut child = cmd.spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            let input = items.join("\n");
            stdin.write_all(input.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            let line = String::from_utf8(output.stdout)?.trim().to_string();
            if let Some(idx_str) = line.split('\t').next()
                && let Ok(idx) = idx_str.parse::<usize>()
            {
                return Ok(Some(idx));
            }
        }
        Ok(None)
    } else {
        // Fallback: strip index prefix for display
        let display: Vec<String> = items
            .iter()
            .filter_map(|s| s.split_once('\t').map(|(_, d)| d.to_string()))
            .collect();
        let selection = dialoguer::Select::new()
            .with_prompt("Select repository")
            .items(&display)
            .interact_opt()?;
        Ok(selection)
    }
}

/// Select from choices, using fzf if available, otherwise dialoguer.
pub fn select_one(choices: &[String], prompt: &str, query: Option<&str>) -> Result<Option<String>> {
    if choices.is_empty() {
        return Ok(None);
    }
    if choices.len() == 1 {
        return Ok(Some(choices[0].clone()));
    }

    if has_fzf() {
        fzf_select(choices, query)
    } else {
        let selection = dialoguer::Select::new()
            .with_prompt(prompt)
            .items(choices)
            .interact_opt()?;
        Ok(selection.map(|i| choices[i].clone()))
    }
}

fn fzf_select(choices: &[String], query: Option<&str>) -> Result<Option<String>> {
    let mut cmd = Command::new("fzf");
    cmd.arg("--ansi")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if let Some(q) = query {
        cmd.args(["--query", q]);
    }

    let mut child = cmd.spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        let input = choices.join("\n");
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        let selected = String::from_utf8(output.stdout)?.trim().to_string();
        if selected.is_empty() {
            Ok(None)
        } else {
            Ok(Some(selected))
        }
    } else {
        Ok(None)
    }
}
