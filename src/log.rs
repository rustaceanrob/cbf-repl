use bdk_kyoto::{logger::NodeMessageHandler, NodeState, Warning};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
pub(crate) struct FileLogger {
    // We need to hold onto this guard for the duration of
    // the program to ensure messages are written to the log file.
    _worker: WorkerGuard
}

impl FileLogger {
    pub(crate) fn new() -> Self {
        // Make daily application logs
        let file_appender = tracing_appender::rolling::daily("logs", "app.log");
        let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

        // We would like console logging
        let console_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(false);

        // As well as file logging
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking_file)
            .with_ansi(false)
            .with_target(false);

        tracing_subscriber::registry()
            .with(console_layer)
            .with(file_layer)
            .init();

        Self {
            _worker: guard
        }
    }
}

// The `NodeMessageHandler` is a trait that allows applications to handle
// events from the node with arbitrary behavior. In this example we 
// write events to both the file and the console. 
impl NodeMessageHandler for FileLogger {
    fn dialog(&self, dialog: String) {
        tracing::info!("{dialog}")
    }

    fn warning(&self, warning: Warning) {
        tracing::warn!("{warning}")
    }

    fn state_changed(&self, state: NodeState) {
        tracing::info!("Sync update: {state}")
    }

    fn connections_met(&self) {
        tracing::info!("All connections have been met")
    }

    fn tx_sent(&self, _txid: bdk_kyoto::Txid) {}

    fn tx_failed(&self, _txid: bdk_kyoto::Txid) {}

    fn blocks_disconnected(&self, _blocks: Vec<u32>) {}

    fn synced(&self, tip: u32) {
        tracing::info!("Synced to height {tip}")
    }
}
