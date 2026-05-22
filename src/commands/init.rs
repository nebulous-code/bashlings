use anyhow::Result;

use crate::container;
use crate::puzzle;

pub fn run(verbose: bool) -> Result<()> {
    let root = puzzle::find_project_root()?;
    let containers_dir = root.join("containers");
    println!(
        "Building base image '{}' from {}",
        container::IMAGE_TAG,
        containers_dir.display()
    );
    container::build_image(&containers_dir, verbose)?;
    println!("Done.");
    Ok(())
}
