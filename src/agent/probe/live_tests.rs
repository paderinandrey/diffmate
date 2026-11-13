use super::{check_flags, check_version};
use crate::agent::claude::ClaudeCli;
use crate::procenv::InheritedEnv;
use std::ffi::OsStr;

#[tokio::test]
#[ignore = "needs the real grok CLI"]
async fn live_help_lists_the_production_flags() {
    let cli = ClaudeCli::new(&InheritedEnv::capture()).unwrap();
    let program = OsStr::new("grok");
    check_version(program, cli.env_vars(), None)
        .await
        .unwrap()
        .unwrap();
    check_flags(program, cli.env_vars(), None)
        .await
        .unwrap()
        .unwrap();
}
