use git2::{BranchType, Repository};

fn main() -> Result<(), failure::Error> {
    let repository = Repository::discover(".")?;

    for branch in repository.branches(Some(BranchType::Local))? {
        let (mut branch, _) = branch?;
        let branch_name = branch.name()?.unwrap_or("branch-name-not-utf8").to_owned();
        match branch.delete() {
            Ok(()) => println!("Deleted {}", &branch_name),
            Err(e) => println!("Error: {}", e.message()),
        }
    }
    Ok(())
}
