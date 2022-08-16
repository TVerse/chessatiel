mod engine_manager;
mod io_handlers;
mod protocol;

use crate::uci::engine_manager::EngineManager;
use crate::uci::io_handlers::{InputHandler, OutputHandler};
use crate::uci::protocol::{IncomingCommand, OutgoingCommand};
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn uci() {
    let (stdin_tx, stdin_rx) = mpsc::unbounded_channel();
    let (stdout_tx, stdout_rx) = mpsc::unbounded_channel();

    let _ = start_stdin_thread(stdin_tx, stdout_tx.clone());
    let _ = start_stdout_thread(stdout_rx);

    let engine_manager = EngineManager::new(stdin_rx, stdout_tx);

    engine_manager.run().await
}

fn start_stdin_thread(
    tx: UnboundedSender<IncomingCommand>,
    tx_err: UnboundedSender<OutgoingCommand>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("stdin".to_string())
        .spawn(move || {
            let stdin = std::io::stdin();
            let mut stdin_lock = stdin.lock();
            let mut input_handler = InputHandler::new(&mut stdin_lock, tx, tx_err);
            loop {
                input_handler.handle_one();
            }
        })
        .unwrap()
}

fn start_stdout_thread(rx: UnboundedReceiver<OutgoingCommand>) -> JoinHandle<()> {
    thread::Builder::new()
        .name("stdout".to_owned())
        .spawn(move || {
            let stdout = std::io::stdout();
            let mut stdout_lock = stdout.lock();
            let mut output_handler = OutputHandler::new(&mut stdout_lock, rx);
            loop {
                output_handler.handle_one();
            }
        })
        .unwrap()
}
