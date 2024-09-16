mod log;

use bdk_kyoto::builder::LightClientBuilder;
use bdk_wallet::*;
use bitcoin::Network;
use log::FileLogger;
use rusqlite::Connection;
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines, Stdin},
    select,
};

const WALLET_FILE: &str = ".bdk-wallet.sqlite";

const RECEIVE: &str = "tr([7d94197e/86'/1'/0']tpubDCyQVJj8KzjiQsFjmb3KwECVXPvMwvAxxZGCP9XmWSopmjW3bCV3wD7TgxrUhiGSueDS1MU5X1Vb1YjYcp8jitXc5fXfdC1z68hDDEyKRNr/0/*)";
const CHANGE: &str = "tr([7d94197e/86'/1'/0']tpubDCyQVJj8KzjiQsFjmb3KwECVXPvMwvAxxZGCP9XmWSopmjW3bCV3wD7TgxrUhiGSueDS1MU5X1Vb1YjYcp8jitXc5fXfdC1z68hDDEyKRNr/1/*)";

// Available commmands in the terminal
const CMD_RECV: &str = "address";
const CMD_BALANCE: &str = "balance";
const CMD_SHUTDOWN: &str = "shutdown";

// Read a line from the terminal input
async fn read_lines(stdin: &mut Lines<BufReader<Stdin>>) -> Option<String> {
    stdin.next_line().await.ok().unwrap_or(None)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up the terminal input
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    // Open a connection to the wallet database
    let mut conn = Connection::open(WALLET_FILE)?;

    // Log events that are issued from the node to the user
    let logger = FileLogger::new();

    tracing::info!("Ready for commands...");

    loop {
        select! {
            lines = read_lines(&mut lines) => {
                if let Some(line) = lines {
                    match line.as_str() {
                        CMD_RECV => {
                            // tracing::info!("Your next address: {}", address);
                        },
                        CMD_BALANCE => {
                            // tracing::info!("Your wallet balance is: {}", balance);
                        },
                        CMD_SHUTDOWN => {
                            return Ok(());
                        }
                        _ => continue
                    }
                }
            }
        }
    }
}
