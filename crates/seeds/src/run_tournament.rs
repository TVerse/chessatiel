use anyhow::{anyhow, Result};
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use itertools::Itertools;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

#[derive(Debug)]
pub struct IdAndFilename {
    name: String,
    id: String,
}

impl FromStr for IdAndFilename {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut split = s.split('=');
        let (name, hash) = split
            .next_tuple()
            .ok_or_else(|| format!("Could not parse {s} into filename=hash"))?;
        Ok(Self {
            name: name.to_owned(),
            id: hash.to_owned(),
        })
    }
}

pub fn run_tournament(hashes: &[IdAndFilename], output_folder: PathBuf) -> Result<()> {
    let _ = std::fs::remove_dir_all(&output_folder);
    std::fs::create_dir(&output_folder)?;
    for IdAndFilename { id, name } in hashes {
        let tempfolder = tempfile::tempdir()?;
        let path = tempfolder.path();
        let repo = builder(Some(id))
            .clone("git@github.com:tverse/chessatiel.git", path)
            .or_else(|e| {
                println!("Error getting branch {id}, assuming it's a commit on main: {e}");
                builder(None).clone("git@github.com:tverse/chessatiel.git", path)
            })?;
        let (object, reference) = repo.revparse_ext(id)?;
        repo.checkout_tree(&object, None)?;
        match reference {
            Some(gref) => repo.set_head(gref.name().unwrap())?,
            None => repo.set_head_detached(object.id())?,
        }

        let mut child = Command::new("cargo")
            .args(["build", "--release", "-p", "chessatiel"])
            .current_dir(path)
            .spawn()?;
        let exit_status = child.wait()?;
        if !exit_status.success() {
            return Err(anyhow!("Cargo returned with error {exit_status}"));
        }
        let binary_path = path.join(Path::new("target/release/chessatiel"));
        let target_path = output_folder.join(name);
        std::fs::copy(binary_path, target_path)?;
    }

    let mut args = vec![
        "-tournament",
        "round-robin",
        "-rounds",
        "30",
        "-openings",
        "file=../openings/openings.pgn",
        "format=pgn",
        "order=random",
        "plies=10",
        "-each",
        "tc=150/1+0.1",
        "restart=on",
        "proto=uci",
        "-concurrency",
        "4",
        "-maxmoves",
        "100",
        "-games",
        "1",
        "-epdout",
        "tournament.epd",
        "-pgnout",
        "tournament.pgn",
        "-srand",
        "1029384756",
        "-recover"
    ]
    .into_iter()
    .map(|s| s.to_owned())
    .collect_vec();
    for IdAndFilename { name, .. } in hashes {
        args.extend(engine_args(name).into_iter());
    }
    println!("{args:?}");
    let mut cutechess_cli = Command::new("cutechess-cli")
        .current_dir(output_folder)
        .args(args)
        .spawn()?;
    let exit_status = cutechess_cli.wait()?;
    if !exit_status.success() {
        return Err(anyhow!("Cutechess-cli returned with error {exit_status}"));
    }

    Ok(())
}

fn builder(branch: Option<&str>) -> RepoBuilder {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username, _allowed_types| {
        Cred::ssh_key(
            username.unwrap(),
            None,
            Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            env::var("SSH_KEY").ok().as_deref(),
        )
    });
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);
    if let Some(branch) = branch {
        builder.branch(branch);
    }
    builder
}

fn engine_args(name: &str) -> Vec<String> {
    vec![
        "-engine".to_owned(),
        format!("name={name}"),
        format!("cmd=./{name}"),
        format!("stderr={name}.log"),
    ]
}
