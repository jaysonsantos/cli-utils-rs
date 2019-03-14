use std::env::{current_dir, set_var, var};
use std::process;

use env_logger::try_init;
use failure::ResultExt;
use git2::{BranchType, Repository};
use log::{debug, error, info, warn};

fn main() -> Result<(), failure::Error> {
    if var("RUST_LOG").is_err() {
        set_var("RUST_LOG", "info");
    }
    try_init()?;
    let repository =
        Repository::discover(current_dir().context("Failed to fetch current directory")?)?;

    fetch_new_data(&repository)?;

    for branch in repository.branches(Some(BranchType::Local))? {
        let (mut branch, _) = branch?;
        let branch_name = if let Some(branch_name) = branch.name()? {
            branch_name.to_owned()
        } else {
            info!("Ignoring branch that could not be decoded to utf-8");
            debug!("Crazy branch name {:?}", branch.name_bytes());
            continue;
        };
        match branch.delete() {
            Ok(()) => info!("Deleted {}", &branch_name),
            Err(e) => error!("{}", e.message()),
        }
    }
    Ok(())
}

fn fetch_new_data(repository: &Repository) -> Result<(), failure::Error> {
    // It is easier to call a subcommand than implement this with libgit :D
    info!("Fetching repository data.");
    let child = process::Command::new("git")
        .current_dir(repository.path())
        .args(&["fetch", "-p"])
        .output()
        .context("Error fetching repository data")?;

    let stdout_return = String::from_utf8_lossy(&child.stdout);
    let stderr_return = String::from_utf8_lossy(&child.stderr);
    if child.status.success() {
        debug!(
            "Process return\nSTDOUT: {:?}\nSTDERR: {:?}",
            stdout_return, stderr_return
        );
    } else {
        warn!(
            "Error running git fetch\nProcess return\nSTDOUT: {:?}\nSTDERR: {:?}",
            stdout_return, stderr_return
        );
    }

    Ok(())
}
