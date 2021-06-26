use log::*;

use beak::{IncomingCommand, OutgoingCommand};
use chessatiel::engine_manager::EngineManager;
use chessatiel::input_handler::InputHandler;
use chessatiel::output_handler::OutputHandler;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use stderrlog::{ColorChoice, Timestamp};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Chessatiel")]
struct Opt {
    #[structopt(long)]
    do_init: bool,
}

fn main() {
    stderrlog::new()
        .verbosity(3)
        .show_module_names(true)
        .color(ColorChoice::Auto)
        .timestamp(Timestamp::Millisecond)
        .init()
        .unwrap();
    info!("Initializing...");

    let opt = Opt::from_args();

    let (stdin_tx, stdin_rx) = mpsc::channel();
    let (stdout_tx, stdout_rx) = mpsc::channel();

    if opt.do_init {
        info!("Sending 'uci' and 'isready' commands for quick init...");
        stdin_tx.send(IncomingCommand::Uci).unwrap();
        stdin_tx.send(IncomingCommand::IsReady).unwrap();
    }

    start_stdin_thread(stdin_tx, stdout_tx.clone());
    start_stdout_thread(stdout_rx);

    let mut engine_manager = EngineManager::new(stdin_rx, stdout_tx);

    engine_manager.start()
}

fn start_stdin_thread(tx: Sender<IncomingCommand>, tx_err: Sender<OutgoingCommand>) {
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
        .unwrap();
}

fn start_stdout_thread(rx: Receiver<OutgoingCommand>) {
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
        .unwrap();
}
