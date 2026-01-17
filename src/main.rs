use std::{env, io, path::PathBuf};

use color_eyre::eyre::{self, OptionExt, bail};

use crate::{domain::Transaction, engine::AccountingEngine};

mod domain;
mod engine;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let input = parse_args()?;

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(input)?;

    let mut engine = AccountingEngine::default();

    for result in reader.deserialize::<Transaction>() {
        let tx = result?;

        // We ignore invalid transactions for simplicity.
        // In a real application we would need to do something about them.
        let _ = engine.handle_transaction(tx);
    }

    let accounts = engine.account_states();

    let mut writer = csv::Writer::from_writer(io::stdout());
    for account in accounts {
        writer.serialize(account)?;
    }

    Ok(())
}

fn parse_args() -> eyre::Result<PathBuf> {
    let mut args = env::args();
    args.next(); // Skip the program name

    let input_path = args
        .next()
        .ok_or_eyre("Please provide the path to the input CSV file.")?;

    if args.next().is_some() {
        bail!("Too many arguments provided. Please provide only the path to the input CSV file.");
    }

    Ok(PathBuf::from(input_path))
}
