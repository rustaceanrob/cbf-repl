use bdk_kyoto::{builder::LightClientBuilder, logger::TraceLogger};
use bdk_wallet::*;
use bitcoin::Network;
use tokio::{io::{AsyncBufReadExt, BufReader, Lines, Stdin}, select};
use rusqlite::Connection;

const RECEIVE: &str = "tr([7d94197e/86'/1'/0']tpubDCyQVJj8KzjiQsFjmb3KwECVXPvMwvAxxZGCP9XmWSopmjW3bCV3wD7TgxrUhiGSueDS1MU5X1Vb1YjYcp8jitXc5fXfdC1z68hDDEyKRNr/0/*)";
const CHANGE: &str = "tr([7d94197e/86'/1'/0']tpubDCyQVJj8KzjiQsFjmb3KwECVXPvMwvAxxZGCP9XmWSopmjW3bCV3wD7TgxrUhiGSueDS1MU5X1Vb1YjYcp8jitXc5fXfdC1z68hDDEyKRNr/1/*)";
const CMD_RECV: &str = "address";
const CMD_BALANCE: &str = "balance";
const CMD_SHUTDOWN: &str = "shutdown";

async fn read_lines(stdin: &mut Lines<BufReader<Stdin>>) -> Option<String> {
    stdin.next_line().await.ok().unwrap_or(None)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    let mut conn = Connection::open(".bdk-wallet.sqlite")?;

    let wallet_opt = Wallet::load()
        .descriptor(KeychainKind::External, Some(RECEIVE))
        .descriptor(KeychainKind::Internal, Some(CHANGE))
        .extract_keys()
        .check_network(Network::Signet)
        .load_wallet(&mut conn)?;

    let mut wallet = match wallet_opt {
        Some(wallet) => wallet,
        None => Wallet::create(RECEIVE, CHANGE)
            .network(Network::Signet)
            .create_wallet(&mut conn)?,
    };

    let (mut node, mut client) = LightClientBuilder::new(&wallet)
        .scan_after(170_000)
        .connections(1)
        .build()?;

    tokio::task::spawn(async move { node.run().await });

    let logger = TraceLogger::new();

    let wallet_update = client.update(&logger).await;

    if let Some(update) = wallet_update {
        wallet.apply_update(update)?;
        wallet.persist(&mut conn)?;
    }

    loop {
        tracing::info!("Awaiting the next command...");
        select! {
            update = client.update(&logger) => {
                if let Some(update) = update {
                    wallet.apply_update(update)?;
                    wallet.persist(&mut conn)?;                    
                }
            },
            lines = read_lines(&mut lines) => {
                if let Some(line) = lines {
                    match line.as_str() {
                        CMD_RECV => {
                            let balance = wallet.reveal_next_address(KeychainKind::External);
                            tracing::info!("Your next address: {}", balance);
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
