use bashlings::container::{self, PodmanRun};
use std::path::Path;

#[test]
fn default_args_set_security_flags() {
    let ws = Path::new("/tmp/ws-x");
    let args = PodmanRun::new(ws)
        .command(["bash", "/puzzle/check.sh"])
        .build_args();
    let joined = args.join(" ");

    assert_eq!(args.first().map(String::as_str), Some("run"));
    assert!(args.contains(&"--rm".to_string()), "missing --rm: {joined}");
    assert!(
        args.contains(&"--network=none".to_string()),
        "missing --network=none: {joined}"
    );
    assert!(
        args.contains(&"--read-only".to_string()),
        "missing --read-only: {joined}"
    );
    assert!(
        args.contains(&"--cap-drop=ALL".to_string()),
        "missing --cap-drop=ALL: {joined}"
    );
    assert!(
        args.contains(&"--security-opt=no-new-privileges".to_string()),
        "missing --security-opt=no-new-privileges: {joined}"
    );
    assert!(
        args.contains(&"--memory=512m".to_string()),
        "missing --memory=512m: {joined}"
    );
    // --cpus is intentionally absent (see container.rs for why).
    assert!(
        !args.contains(&"--cpus=1".to_string()),
        "--cpus=1 should be omitted — see container.rs rationale: {joined}"
    );
    assert!(
        args.contains(&"--pids-limit=256".to_string()),
        "missing --pids-limit=256: {joined}"
    );
    assert!(
        joined.contains("/tmp/ws-x:/puzzle"),
        "workspace mount missing: {joined}"
    );
    assert!(
        args.contains(&container::IMAGE_TAG.to_string()),
        "image tag missing: {joined}"
    );
}

#[test]
fn allow_network_drops_network_none() {
    let ws = Path::new("/tmp/ws-x");
    let args = PodmanRun::new(ws)
        .allow_network(true)
        .command(["bash", "/puzzle/check.sh"])
        .build_args();
    assert!(
        !args.contains(&"--network=none".to_string()),
        "--network=none must not be present when allow_network is true"
    );
}

#[test]
fn interactive_adds_it_flags() {
    let ws = Path::new("/tmp/ws-x");
    let args = PodmanRun::new(ws)
        .interactive(true)
        .command(["bash"])
        .build_args();
    assert!(args.contains(&"-i".to_string()));
    assert!(args.contains(&"-t".to_string()));
}
