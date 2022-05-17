use std::{fs::File, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Sequence};

/// Generates invoices for my freelance activity
#[derive(Parser, Debug)]
#[clap(author = "Alexandre Borghi <borghi.alexandre.12@gmail.com>")]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the invoice data YAML file
    path: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let f = File::open(args.path)?;
    let invoice: Invoice =
        serde_yaml::from_reader(f).expect("Error: Failed to deserialize invoice from file");

    println!("{:?}", invoice);

    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Invoice {
    number: String,
    issued: String,  // date
    shipped: String, // date

    company: Mapping,
    client: Mapping,

    detail: Sequence,
}
