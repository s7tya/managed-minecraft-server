use std::{fs::create_dir_all, path::Path};

use download::download_paper_jar;

pub mod download;

fn main() -> anyhow::Result<()> {
    create_dir_all(Path::new("dist/"))?;
    download_paper_jar(Path::new("dist/paper.jar"))?;
    Ok(())
}
