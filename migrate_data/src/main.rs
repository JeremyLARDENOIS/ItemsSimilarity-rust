mod dump;
mod store;
mod models;
mod tokenize;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Dump data
    Dump,
    /// Store data
    Store,
    /// Tokenize data
    Tokenize,
}


#[async_std::main]
async fn main() {
    match Cli::parse().mode {
        Mode::Dump => {
            println!("Dumping data");
            dump::dump().await.expect("Failed to dump data");
        }
        Mode::Store => {
            println!("Storing data");
            store::store();
        }
        Mode::Tokenize => {
            println!("Tokenizing data");
            tokenize::main();
        }
    }
    println!("Done");
}
