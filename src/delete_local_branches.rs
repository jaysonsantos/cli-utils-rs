use env_logger::Builder;
use git2::{BranchType, Repository};
use log::{debug, error, info, LevelFilter};
use std::process;

fn main() -> Result<(), failure::Error> {
    Builder::new().filter_level(LevelFilter::Info).try_init()?;
    let repository = Repository::discover(".")?;

    fetch_new_data(&repository)?;

    for branch in repository.branches(Some(BranchType::Local))? {
        let (mut branch, _) = branch?;
        let branch_name = branch.name()?.unwrap_or("branch-name-not-utf8").to_owned();
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
        .output()?;

    debug!(
        "Process return\nSTDOUT: {:?}\nSTDERR: {:?}",
        String::from_utf8(child.stdout).unwrap(),
        String::from_utf8(child.stderr).unwrap()
    );
    Ok(())
}
