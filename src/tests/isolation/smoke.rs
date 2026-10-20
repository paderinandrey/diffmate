use super::contract::control_fired;
use super::endpoint::FakeAnthropic;
use super::fixture::MaliciousPr;
use super::harness::{run_control, run_isolated, run_open_control};
use super::inspect::{bodies_contain, tool_names};

#[tokio::test]
#[ignore = "needs the real claude CLI"]
async fn harness_smoke() {
    assert!(control_fired("claude_md"));
    let secondary = FakeAnthropic::start().await;
    let pr = MaliciousPr::build(&secondary.url);
    for name in [
        "control",
        "open",
        "isolated",
        "agents-control",
        "agents-isolated",
    ] {
        let endpoint = FakeAnthropic::start().await;
        if name.starts_with("agents") {
            pr.set_claude_md(false);
        }
        match name {
            "control" | "agents-control" => run_control(&pr, &endpoint).await,
            "open" => run_open_control(&pr, &endpoint).await,
            _ => run_isolated(&pr, &endpoint).await,
        };
        if name.starts_with("agents") {
            pr.set_claude_md(true);
        }
        let seen = [endpoint.take(), secondary.take()].concat();
        let paths: Vec<_> = seen
            .iter()
            .map(|r| format!("{} {} {}", r.method, r.path, r.body.len()))
            .collect();
        eprintln!(
            "{name}: {paths:?} markers={:?} secondary={}",
            pr.fired(),
            secondary.connections()
        );
        eprintln!("{name}: tools={:?}", tool_names(&seen));
        let found: Vec<_> = pr
            .canaries
            .iter()
            .filter(|(_, v)| bodies_contain(&seen, v))
            .map(|(k, _)| *k)
            .collect();
        eprintln!("{name}: canaries seen={found:?}");
    }
}
