use bashlings::puzzle;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn discovers_and_validates_all_puzzles() {
    let root = repo_root();
    let puzzles = puzzle::discover(&root).expect("discover puzzles");
    assert!(
        !puzzles.is_empty(),
        "no puzzles found under puzzles/ — expected starter set"
    );

    // Ids must be unique.
    let mut ids: Vec<_> = puzzles.iter().map(|p| p.meta.id.clone()).collect();
    ids.sort();
    let original = ids.clone();
    ids.dedup();
    assert_eq!(ids, original, "duplicate puzzle ids");

    // Every puzzle's required scripts exist (load() already enforces, but be explicit).
    for p in &puzzles {
        assert!(
            p.dir.join("setup.sh").exists(),
            "missing setup.sh for {}",
            p.meta.id
        );
        assert!(
            p.dir.join("check.sh").exists(),
            "missing check.sh for {}",
            p.meta.id
        );
        assert!(
            p.dir.join("solution.sh").exists(),
            "missing solution.sh for {}",
            p.meta.id
        );
        assert!(
            p.dir.join("README.md").exists(),
            "missing README.md for {}",
            p.meta.id
        );
    }
}
