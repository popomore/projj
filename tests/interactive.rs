use std::fs;

fn bin_path() -> String {
    let path = assert_cmd::cargo::cargo_bin("projj");
    path.to_string_lossy().to_string()
}

fn spawn_projj(home: &std::path::Path, args: &str) -> rexpect::session::PtySession {
    let cmd = format!("env HOME={} {} {args}", home.display(), bin_path());
    rexpect::spawn(&cmd, Some(10000)).unwrap()
}

fn spawn_projj_no_color(home: &std::path::Path, args: &str) -> rexpect::session::PtySession {
    let cmd = format!(
        "env HOME={} NO_COLOR=1 {} {args}",
        home.display(),
        bin_path()
    );
    rexpect::spawn(&cmd, Some(10000)).unwrap()
}

fn setup_env(base: &std::path::Path) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let projj_dir = home.path().join(".projj");
    fs::create_dir_all(&projj_dir).unwrap();
    let config = format!("base = \"{}\"\nplatform = \"github.com\"\n", base.display());
    fs::write(projj_dir.join("config.toml"), config).unwrap();
    home
}

fn create_repo(base: &std::path::Path, host: &str, owner: &str, repo: &str) {
    let repo_path = base.join(host).join(owner).join(repo).join(".git");
    fs::create_dir_all(repo_path).unwrap();
}

// ── init ──

#[test]
fn test_init_interactive() {
    let home = tempfile::tempdir().unwrap();
    let base_dir = home.path().join("my-repos");
    fs::create_dir_all(&base_dir).unwrap();

    let mut p = spawn_projj(home.path(), "init");

    p.exp_string("Set base directory").unwrap();
    p.send_line(&base_dir.to_string_lossy()).unwrap();

    p.exp_string("Default platform").unwrap();
    p.send_line("github.com").unwrap();

    p.exp_string("Config saved").unwrap();
    p.exp_string("shell-setup").unwrap();

    p.exp_eof().unwrap();

    assert!(home.path().join(".projj/config.toml").exists());
}

#[test]
fn test_init_existing_config() {
    let base = tempfile::tempdir().unwrap();
    let home = setup_env(base.path());

    let mut p = spawn_projj(home.path(), "init");

    p.exp_string("Config already exists").unwrap();
    p.exp_string("shell-setup").unwrap();
    p.exp_eof().unwrap();
}

// ── remove ──

#[test]
fn test_remove_confirm() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "testowner", "testrepo");
    let home = setup_env(base.path());

    let mut p = spawn_projj(home.path(), "remove testrepo");

    p.exp_string("Will remove").unwrap();
    p.exp_string("cannot be undone").unwrap();
    p.exp_string("testowner/testrepo").unwrap();

    p.send_line("testowner/testrepo").unwrap();
    p.exp_string("Removed").unwrap();
    p.exp_eof().unwrap();

    assert!(!base.path().join("github.com/testowner/testrepo").exists());
}

#[test]
fn test_remove_wrong_name() {
    let base = tempfile::tempdir().unwrap();
    create_repo(base.path(), "github.com", "testowner", "testrepo");
    let home = setup_env(base.path());

    let mut p = spawn_projj(home.path(), "remove testrepo");

    p.exp_string("Will remove").unwrap();
    p.send_line("wrong-name").unwrap();

    p.exp_string("Cancelled").unwrap();
    p.exp_eof().unwrap();

    assert!(base.path().join("github.com/testowner/testrepo").exists());
}
