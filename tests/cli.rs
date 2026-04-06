use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Create a temp config dir with a `config.toml` and return the dir.
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

// ── projj add (local move) ──

#[test]
fn test_add_local_repo() {
    let base = tempfile::tempdir().unwrap();
    let local = tempfile::tempdir().unwrap();

    // Create a fake local git repo with remote
    let repo_dir = local.path().join("myrepo");
    fs::create_dir_all(repo_dir.join(".git")).unwrap();
    // Create a minimal git config with remote
    fs::create_dir_all(repo_dir.join(".git")).unwrap();
    fs::write(
        repo_dir.join(".git/config"),
        "[remote \"origin\"]\n\turl = git@github.com:testowner/testrepo.git\n",
    )
    .unwrap();
    // git config --get needs a proper git repo, so init it
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "git@github.com:testowner/testrepo.git",
        ])
        .current_dir(&repo_dir)
        .output()
        .unwrap();

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
        .args(["add", &repo_dir.to_string_lossy()])
        .assert()
        .success()
        .stderr(predicate::str::contains("Moving"))
        .stdout(predicate::str::contains("github.com/testowner/testrepo"));

    // Original should be moved
    assert!(!repo_dir.exists());
    // Target should exist
    assert!(
        base.path()
            .join("github.com/testowner/testrepo/.git")
            .exists()
    );
}

// ── projj list (multiple bases) ──

#[test]
fn test_list_multiple_bases() {
    let base1 = tempfile::tempdir().unwrap();
    let base2 = tempfile::tempdir().unwrap();
    create_repo(base1.path(), "github.com", "a", "repo1");
    create_repo(base2.path(), "gitlab.com", "b", "repo2");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config_content = format!(
        "base = [\"{}\", \"{}\"]\nplatform = \"github.com\"\n",
        base1.path().display(),
        base2.path().display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    // Pretty list should show group headers
    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("a/repo1"))
        .stdout(predicate::str::contains("b/repo2"))
        .stdout(predicate::str::contains("Total: 2 repositories"));

    // Raw list
    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["list", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("github.com/a/repo1"))
        .stdout(predicate::str::contains("gitlab.com/b/repo2"));
}

// ── projj find (case insensitive) ──

#[test]
fn test_find_case_insensitive() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "Owner", "MyRepo");

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
        .args(["find", "myrepo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Owner/MyRepo"));
}

// ── projj add with hooks ──

#[test]
fn test_add_existing_with_post_add_hook() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "popomore", "projj");

    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();

    let marker = base.path().join("hook_ran");
    let config_content = format!(
        r#"base = "{}"
platform = "github.com"

[[hooks]]
event = "post_add"
command = "touch {}"
"#,
        base.path().display(),
        marker.display()
    );
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["add", "popomore/projj"])
        .assert()
        .success();

    assert!(marker.exists(), "post_add hook should have run");
}

// ── projj run with hooks dir ──

#[test]
fn test_run_from_hooks_dir() {
    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    let hooks_dir = projj_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    // Create a hook script
    let script_path = hooks_dir.join("greet");
    fs::write(&script_path, "#!/bin/bash\necho hello-from-hook").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let config_content = "base = \"/tmp\"\nplatform = \"github.com\"\n";
    fs::write(projj_dir.join("config.toml"), config_content).unwrap();

    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["run", "greet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello-from-hook"));
}

// ── no config for various commands ──

#[test]
fn test_no_config_find() {
    let home = tempfile::tempdir().unwrap();
    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["find", "foo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("projj init"));
}

#[test]
fn test_no_config_add() {
    let home = tempfile::tempdir().unwrap();
    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["add", "owner/repo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("projj init"));
}

#[test]
fn test_no_config_run() {
    let home = tempfile::tempdir().unwrap();
    Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["run", "echo hi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("projj init"));
}

// ── NO_COLOR ──

#[test]
fn test_list_no_color() {
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

    let output = Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .env("NO_COLOR", "1")
        .args(["list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Should not contain any ANSI escape codes
    assert!(
        !stdout.contains("\x1b["),
        "output should not contain ANSI codes"
    );
    assert!(stdout.contains("popomore/projj"));
    assert!(stdout.contains("Total:"));
}

#[test]
fn test_find_no_color() {
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

    let output = Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .env("NO_COLOR", "1")
        .args(["find", "projj"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        !stdout.contains("\x1b["),
        "stdout should not contain ANSI codes"
    );
    assert!(
        !stderr.contains("\x1b["),
        "stderr should not contain ANSI codes"
    );
    assert!(stdout.contains("github.com/popomore/projj"));
}

#[test]
fn test_list_no_color_raw_unchanged() {
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

    // --raw should be identical with and without NO_COLOR
    let with_color = Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .args(["list", "--raw"])
        .output()
        .unwrap();

    let without_color = Command::cargo_bin("projj")
        .unwrap()
        .env("HOME", home.path())
        .env("NO_COLOR", "1")
        .args(["list", "--raw"])
        .output()
        .unwrap();

    assert_eq!(with_color.stdout, without_color.stdout);
}
