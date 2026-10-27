use super::fixture::poison::{Poison, control, poison};
use super::fixture::{path_var, upstream};
use super::{Git, GitCli};
use crate::procenv::InheritedEnv;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn poisoned_cli(root: &Path, poison: &Poison) -> GitCli {
    let (home, xdg) = (root.join("home"), root.join("xdg"));
    fs::create_dir_all(xdg.join("git")).unwrap();
    fs::create_dir_all(&home).unwrap();
    fs::copy(&poison.config, home.join(".gitconfig")).unwrap();
    fs::copy(&poison.config, xdg.join("git/config")).unwrap();
    fs::create_dir_all(home.join(".config/git")).unwrap();
    for attributes in [
        home.join(".config/git/attributes"),
        xdg.join("git/attributes"),
    ] {
        fs::write(attributes, "* -diff\n").unwrap();
    }
    let monitor = format!("'core.fsmonitor'='{}'", poison.fsmonitor.display());
    let pairs = [
        ("PATH", path_var()),
        ("HOME", home.into()),
        ("XDG_CONFIG_HOME", xdg.into()),
        ("GIT_CONFIG_GLOBAL", poison.config.clone().into()),
        ("GIT_CONFIG_PARAMETERS", monitor.into()),
        ("GIT_CONFIG_COUNT", "1".into()),
        ("GIT_CONFIG_KEY_0", "core.hooksPath".into()),
        ("GIT_CONFIG_VALUE_0", poison.hooks.clone().into()),
        ("GIT_DIR", root.join("decoy.git").into()),
    ];
    let pairs = pairs.into_iter().map(|(k, v)| (k.into(), v)).collect();
    GitCli::new(&InheritedEnv::from_pairs(pairs))
}

#[tokio::test]
async fn poisoned_configuration_runs_in_the_control_and_never_in_diffmate() {
    let tmp = TempDir::new().unwrap();
    let (root, poison) = (tmp.path(), poison(tmp.path()));
    let up = upstream(root);
    control(&poison, root, &up);

    let cli = poisoned_cli(root, &poison);
    let (repo, tree) = (root.join("r.git"), root.join("tree"));
    let hooks = root.join("empty");
    cli.ensure_clone(&repo, up.path.to_str().unwrap(), &hooks)
        .await
        .unwrap();
    cli.fetch_pr(&repo, 1, "main").await.unwrap();
    assert_eq!(
        cli.rev_parse(&repo, "refs/diffmate/pull/1/head")
            .await
            .unwrap(),
        up.head
    );
    assert_eq!(
        cli.merge_base(&repo, &up.base, &up.head).await.unwrap(),
        up.base
    );
    cli.worktree_add(&repo, &tree, &up.head).await.unwrap();
    GitCli::run(cli.command(Some(&tree), ["status"]))
        .await
        .unwrap();
    let diff = cli.diff(&repo, &up.base, &up.head).await.unwrap();
    assert!(String::from_utf8_lossy(&diff).contains("+two"));
    cli.worktree_remove(&repo, &tree).await.unwrap();
    cli.worktree_prune(&repo).await.unwrap();

    assert_eq!(fs::read_dir(&poison.markers).unwrap().count(), 0);
}

#[test]
fn command_environment_carries_no_inherited_git_variable() {
    let tmp = TempDir::new().unwrap();
    let poison = poison(tmp.path());
    let command = poisoned_cli(tmp.path(), &poison).command(None, ["status"]);
    let mut git_vars: Vec<_> = command
        .as_std()
        .get_envs()
        .filter(|(name, _)| name.to_string_lossy().starts_with("GIT_"))
        .map(|(name, value)| {
            format!(
                "{}={}",
                name.to_string_lossy(),
                value.unwrap().to_string_lossy()
            )
        })
        .collect();
    git_vars.sort();
    let expected = [
        "GIT_CONFIG_GLOBAL=/dev/null",
        "GIT_CONFIG_NOSYSTEM=1",
        "GIT_TERMINAL_PROMPT=0",
    ];
    assert_eq!(git_vars, expected);
}
