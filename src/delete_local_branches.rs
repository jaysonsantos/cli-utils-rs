use std::env::{current_dir, set_var, var};
use std::process;

use color_eyre::eyre::{Result, WrapErr};
use env_logger::try_init;
use git2::{BranchType, Repository};
use tracing::{debug, error, info, warn};

fn main() -> Result<()> {
    if var("RUST_LOG").is_err() {
        set_var("RUST_LOG", "info");
    }
    try_init()?;
    let repository =
        Repository::discover(current_dir().wrap_err("Failed to fetch current directory")?)?;

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

fn fetch_new_data(repository: &Repository) -> Result<()> {
    // It is easier to call a subcommand than implement this with libgit :D
    info!("Fetching repository data.");
    let child = process::Command::new("git")
        .current_dir(repository.path())
        .args(&["fetch", "-p"])
        .output()
        .wrap_err("Error fetching repository data")?;

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
