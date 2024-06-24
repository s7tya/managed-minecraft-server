use std::{fs::File, io::Write, path::Path};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize, Debug)]
struct BuildsResponse {
    project_id: String,
    project_name: String,
    version: String,
    builds: Vec<Build>,
}

#[derive(Deserialize, Debug)]
struct Build {
    build: usize,
    channel: String,
    promoted: bool,
    time: DateTime<Utc>,
    changes: Vec<BuildChanges>,
    downloads: BuildDownloads,
}

#[derive(Deserialize, Debug)]
struct BuildChanges {
    commit: String,
    summary: String,
    message: String,
}

#[derive(Deserialize, Debug)]
struct BuildDownloads {
    application: BuildDownloadsApplication,
}

#[derive(Deserialize, Debug)]
struct BuildDownloadsApplication {
    name: String,
    sha256: String,
}

fn is_version_stable(project: &str, version: &str) -> anyhow::Result<bool> {
    let resp: BuildsResponse = reqwest::blocking::get(format!(
        "https://api.papermc.io/v2/projects/{project}/versions/{version}/builds"
    ))?
    .json()?;
    let release_channel = &resp.builds.last().unwrap().channel;
    Ok(release_channel == "default")
}

#[derive(Deserialize)]
struct ProjectResponse {
    project_id: String,
    project_name: String,
    version_groups: Vec<String>,
    versions: Vec<String>,
}

fn get_latest_stable_version(project: &str) -> anyhow::Result<(String, String)> {
    let resp: ProjectResponse =
        reqwest::blocking::get(format!("https://api.papermc.io/v2/projects/{project}"))?.json()?;
    let latest_stable_version = &resp
        .versions
        .iter()
        .rev()
        .find(|version| is_version_stable(project, version).unwrap())
        .unwrap();

    let builds: BuildsResponse = reqwest::blocking::get(format!(
        "https://api.papermc.io/v2/projects/{project}/versions/{latest_stable_version}/builds"
    ))?
    .json()?;
    let build = builds.builds.last().unwrap();

    let url = format!(
        "https://api.papermc.io/v2/projects/{}/versions/{}/builds/{}/downloads/{}",
        project, latest_stable_version, build.build, build.downloads.application.name
    );

    Ok((url, build.downloads.application.sha256.to_owned()))
}

pub fn download_paper_jar(path: &Path) -> anyhow::Result<()> {
    let (url, sha256) = get_latest_stable_version("paper")?;
    let jar_bytes = reqwest::blocking::get(url)?.bytes()?;
    if sha256 != format!("{:x}", Sha256::digest(&jar_bytes)) {
        return Err(anyhow::anyhow!("Invalid jar hash256"));
    }

    let mut file = File::create(path)?;
    file.write_all(&jar_bytes)?;

    Ok(())
}
