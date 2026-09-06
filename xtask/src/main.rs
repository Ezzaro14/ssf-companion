use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("build-data") => build_data(),
        Some(other) => bail!("unknown task `{other}`; try `build-data`"),
        None => {
            eprintln!("usage: cargo xtask build-data");
            Ok(())
        }
    }
}

fn build_data() -> Result<()> {
    let t = Instant::now();
    let data = ssf_data::import_from_dump(
        Path::new("data/base_items.json"),
        Path::new("data/mods.json"),
    )
    .context("importing the dump")?;
    println!(
        "json parse     {:>5}ms   {} modifiers",
        t.elapsed().as_millis(),
        data.mods.len()
    );

    let t = Instant::now();
    let bytes = ssf_data::snapshot::write(&data, Path::new("assets/index.bin"))?;
    println!(
        "snapshot write {:>5}ms   {:.1} MB",
        t.elapsed().as_millis(),
        bytes as f64 / 1e6
    );

    Ok(())
}
