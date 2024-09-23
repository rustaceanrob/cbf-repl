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
    let mut conn = Connection::open(".bdk-wallet.sqlite")?;

    // Attempt to load the wallet from the database connection.
    let wallet_opt = Wallet::load()
        .descriptor(KeychainKind::External, Some(RECEIVE))
        .descriptor(KeychainKind::Internal, Some(CHANGE))
        .check_network(Network::Signet)
        .load_wallet(&mut conn)?;

    let mut wallet = match wallet_opt {
        // We found an existing wallet
        Some(wallet) => wallet,
        // No wallet exists, so we create one here
        None => Wallet::create(RECEIVE, CHANGE)
            .network(Network::Signet)
            .create_wallet(&mut conn)?,
    };

    let (node, mut client) = LightClientBuilder::new(&wallet)
        // When recovering a wallet, specify the height to start scanning
        .scan_after(170_000)
        // The number of remote connections to maintain
        .connections(1)
        .build()?;

    // Run the node on a separate task
    tokio::task::spawn(async move { node.run().await });

    // Log events that are issued from the node to the user
    let logger = FileLogger::new();

    // Wait for an update for the wallet from the node
    let wallet_update = client.update(&logger).await;

    // Apply the update and write it to the database
    if let Some(update) = wallet_update {
        wallet.apply_update(update)?;
        wallet.persist(&mut conn)?;
    }

    loop {
        tracing::info!("Awaiting the next command...");
        select! {
            // Wait for new blocks and apply any updates
            update = client.update(&logger) => {
                if let Some(update) = update {
                    wallet.apply_update(update)?;
                    wallet.persist(&mut conn)?;
                }
            },
            // Wait for a command from the user
            lines = read_lines(&mut lines) => {
                if let Some(line) = lines {
                    match line.as_str() {
                        CMD_RECV => {
                            let balance = wallet.reveal_next_address(KeychainKind::External);
                            tracing::info!("Your next address: {}", balance);
                            wallet.persist(&mut conn)?;
                        },
                        CMD_BALANCE => {
                            let balance = wallet.balance().total().to_sat();
                            tracing::info!("Your wallet balance is: {}", balance);
                        },
                        CMD_SHUTDOWN => {
                            client.shutdown().await?;
                            return Ok(());
                        }
                        _ => continue
                    }
                }
            }
        }
    }
}
