use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use anyhow::{anyhow, Result};
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use itertools::Itertools;

#[derive(Debug)]
pub struct HashAndFilename {
    filename: String,
    hash: String,
}

impl FromStr for HashAndFilename {
    type Err =String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut split = s.split("=");
        let (filename, hash) = split.next_tuple().ok_or_else(||format!("Could not parse {s} into filename=hash"))?;
        Ok(Self {
            filename: filename.to_owned(),
            hash: hash.to_owned(),
        })
    }
}

pub fn run_tournament(hashes: &[HashAndFilename], output_folder: PathBuf) -> Result<()> {
    let mut repo = {
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
        builder
    };
    for HashAndFilename {hash, filename }in hashes {
        let tempfolder = tempfile::tempdir()?;
        let path = tempfolder.path();
        let repo = repo.clone("git@github.com:tverse/chessatiel.git", path)?;
        let (object, reference) = repo.revparse_ext(hash)?;
        repo.checkout_tree(&object, None)?;
        match reference {
            Some(gref) => repo.set_head(gref.name().unwrap())?,
            None => repo.set_head_detached(object.id())?,
        }

        let mut child = Command::new("cargo").args(["build", "--release"]).current_dir(path).spawn()?;
        let exit_status = child.wait()?;
        if !exit_status.success() {
            return Err(anyhow!("Child process returned with error {exit_status}"))
        }
        let binary_path = path.join(Path::new("target/release/chessatiel"));
        let target_path = output_folder.join(filename);
        std::fs::copy(binary_path, target_path)?;
    }

    Ok(())
}
