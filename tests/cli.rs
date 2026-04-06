use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Create a temp config dir with a config.toml and return (dir, config_path).
fn setup_config(base_dir: &std::path::Path) -> TempDir {
    let config_dir = tempfile::tempdir().unwrap();
    let config_content = format!(
        r#"base = "{}"
platform = "github.com"

[scripts]
hello = "echo hello"

[[hooks]]
event = "post_add"
command = "true"
"#,
        base_dir.display()
    );
    fs::write(config_dir.path().join("config.toml"), config_content).unwrap();
    config_dir
}

/// Create a fake repo structure: base/host/owner/repo/.git
fn create_repo(base: &std::path::Path, host: &str, owner: &str, repo: &str) {
    let repo_path = base.join(host).join(owner).join(repo).join(".git");
    fs::create_dir_all(repo_path).unwrap();
}

fn projj_cmd(config_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("projj").unwrap();
    cmd.env("HOME", config_dir.parent().unwrap_or(config_dir));
    // Override HOME so config_dir() points to our temp dir
    // config_dir is $HOME/.projj, so HOME = config_dir/..
    cmd
}

// ── projj list ──

#[test]
fn test_list_raw() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "popomore", "projj");
    create_repo(base.path(), "github.com", "SeeleAI", "agent");

    let config_dir = setup_config(base.path());
    // HOME needs to be parent of .projj
    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    fs::copy(
        config_dir.path().join("config.toml"),
        projj_dir.join("config.toml"),
    )
    .unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["list", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("github.com/popomore/projj"))
        .stdout(predicate::str::contains("github.com/SeeleAI/agent"));
}

#[test]
fn test_list_pretty() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "popomore", "projj");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = \"{}\"\nplatform = \"github.com\"\n",
        base.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("popomore/projj"))
        .stdout(predicate::str::contains("Total: 1 repositories"));
}

// ── projj find ──

#[test]
fn test_find_single_match() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "popomore", "projj");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = \"{}\"\nplatform = \"github.com\"\n",
        base.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["find", "projj"])
        .assert()
        .success()
        .stdout(predicate::str::contains("github.com/popomore/projj"));
}

#[test]
fn test_find_no_match() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "popomore", "projj");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = \"{}\"\nplatform = \"github.com\"\n",
        base.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["find", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No repository found"));
}

// ── projj run ──

#[test]
fn test_run_inline_command() {
    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = "base = \"/tmp\"\nplatform = \"github.com\"\n";
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["run", "echo hello-projj"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello-projj"));
}

#[test]
fn test_run_named_script() {
    let base = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = \"{}\"\nplatform = \"github.com\"\n\n[scripts]\nhello = \"echo from-script\"\n",
        base.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["run", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("from-script"));
}

#[test]
fn test_run_all() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "a", "repo1");
    create_repo(base.path(), "github.com", "b", "repo2");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = \"{}\"\nplatform = \"github.com\"\n",
        base.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["run", "echo hi", "--all"])
        .assert()
        .success()
        .stderr(predicate::str::contains("[1/2]"))
        .stderr(predicate::str::contains("[2/2]"));
}

#[test]
fn test_run_all_with_match() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "teamA", "repo1");
    create_repo(base.path(), "github.com", "teamB", "repo2");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = \"{}\"\nplatform = \"github.com\"\n",
        base.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["run", "echo hi", "--all", "--match", "teamA"])
        .assert()
        .success()
        .stderr(predicate::str::contains("[1/1]"))
        .stderr(predicate::str::contains("teamA/repo1"));
}

#[test]
fn test_run_invalid_match_regex() {
    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = "base = \"/tmp\"\nplatform = \"github.com\"\n";
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["run", "echo hi", "--all", "--match", "[invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid --match pattern"));
}

// ── projj (no config) ──

#[test]
fn test_no_config() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("projj init"));
}

// ── projj shell-setup ──

#[test]
fn test_shell_setup_zsh() {
    Command::cargo_bin("projj")
        .unwrap()
        .args(["shell-setup", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef _projj projj"))
        .stdout(predicate::str::contains("p() {"));
}

#[test]
fn test_shell_setup_bash() {
    Command::cargo_bin("projj")
        .unwrap()
        .args(["shell-setup", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("p() {"));
}

#[test]
fn test_shell_setup_fish() {
    Command::cargo_bin("projj")
        .unwrap()
        .args(["shell-setup", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("function p"));
}

// ── projj --help / --version ──

#[test]
fn test_help() {
    Command::cargo_bin("projj")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage git repositories"));
}

#[test]
fn test_version() {
    Command::cargo_bin("projj")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("projj 3."));
}

// ── projj add (existing repo) ──

#[test]
fn test_add_existing_repo() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "popomore", "projj");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = \"{}\"\nplatform = \"github.com\"\n",
        base.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["add", "popomore/projj"])
        .assert()
        .success()
        .stderr(predicate::str::contains("already exists"))
        .stdout(predicate::str::contains("github.com/popomore/projj"));
}
