use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;

/// Check if a command exists in PATH.
fn has_command(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

// ── fzf ──

pub fn has_fzf() -> bool {
    has_command("fzf")
}

/// Run fzf with a list of choices and optional query.
/// Returns the selected item, or None if cancelled.
pub fn fzf_select(choices: &[String], query: Option<&str>) -> Result<Option<String>> {
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
        // fzf returns non-zero when user cancels (Esc/Ctrl-C)
        Ok(None)
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
        // Fallback to dialoguer
        let selection = dialoguer::Select::new()
            .with_prompt(prompt)
            .items(choices)
            .interact_opt()?;
        Ok(selection.map(|i| choices[i].clone()))
    }
}
