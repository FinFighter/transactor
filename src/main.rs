mod account;
mod error;
mod manager;
mod parse;

use manager::Manager;
use std::env;
use std::process::ExitCode;

#[inline]
fn execute(file: &str, mut manager: Manager) -> Result<(), error::TransactorError> {
    parse::load_data(file, &mut manager)?;
    parse::unload_data(manager)?;
    Ok(())
}

fn main() -> ExitCode {
    let file = match env::args().nth(1) {
        Some(file) => file,
        None => {
            eprintln!("Error: Missing csv file parameter");
            return ExitCode::FAILURE;
        }
    };

    let manager = Manager::new();

    if let Err(error) = execute(&file, manager) {
        eprintln!("Fatal Error: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
