use log::*;

use beak::{IncomingCommand, InfoPayload, OutgoingCommand, UciParser};
use chessatiel::engine_manager::EngineManager;
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
        .name("stdin".to_owned())
        .spawn(move || {
            let parser = UciParser::new();
            let stdin = std::io::stdin();
            let mut buf = String::with_capacity(100);
            loop {
                buf.clear();
                let read = stdin.read_line(&mut buf).unwrap();
                if read != 0 {
                    let parsed = parser.parse(&buf);
                    match parsed {
                        Ok(cmd) => {
                            info!("Got command {}", cmd);
                            tx.send(cmd).unwrap()
                        }
                        Err(err) => {
                            let error_text =
                                format!("Could not parse UCI input '{}': {}", buf, err);
                            warn!("{}", error_text);
                            tx_err
                                .send(OutgoingCommand::Info(InfoPayload::String(error_text)))
                                .unwrap();
                        }
                    }
                }
            }
        })
        .unwrap();
}

fn start_stdout_thread(rx: Receiver<OutgoingCommand>) {
    thread::Builder::new()
        .name("stdout".to_owned())
        .spawn(move || loop {
            let received = rx.recv().unwrap();
            println!("{}", received)
        })
        .unwrap();
}
