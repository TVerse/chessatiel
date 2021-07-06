use beak::{IncomingCommand, InfoPayload, OutgoingCommand, UciParser};
use std::io::BufRead;
use std::sync::mpsc::Sender;

use log::info;
use log::warn;

pub struct InputHandler<'a, I>
where
    I: BufRead,
{
    stdin: &'a mut I,
    tx: Sender<IncomingCommand>,
    tx_err: Sender<OutgoingCommand>,
    uci_parser: UciParser,
    buf: String,
}
impl<'a, I> InputHandler<'a, I>
where
    I: BufRead,
{
    pub fn new(
        stdin: &'a mut I,
        tx: Sender<IncomingCommand>,
        tx_err: Sender<OutgoingCommand>,
    ) -> Self {
        Self {
            stdin,
            tx,
            tx_err,
            uci_parser: UciParser::new(),
            buf: String::with_capacity(100),
        }
    }

    pub fn handle_one(&mut self) {
        self.buf.clear();
        let read = self.stdin.read_line(&mut self.buf).unwrap();
        if read != 0 {
            let parsed = self.uci_parser.parse(&self.buf);
            match parsed {
                Ok(cmd) => {
                    info!("Got command {}", cmd);
                    self.tx.send(cmd).unwrap()
                }
                Err(err) => {
                    let error_text = format!("Could not parse UCI input '{}': {}", self.buf, err);
                    warn!("{}", error_text);
                    self.tx_err
                        .send(OutgoingCommand::Info(
                            InfoPayload::new().with_string(error_text),
                        ))
                        .unwrap();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::sync::mpsc;
    use std::sync::mpsc::TryRecvError;

    #[test]
    fn parse_and_send_from_stdin() {
        let (stdin_tx, stdin_rx) = mpsc::channel();
        let (stdout_tx, stdout_rx) = mpsc::channel();

        let stdin = "uci\n";

        let mut reader = BufReader::new(stdin.as_bytes());
        let mut input_handler = InputHandler::new(&mut reader, stdin_tx, stdout_tx);

        input_handler.handle_one();

        assert_eq!(stdin_rx.try_recv().unwrap(), IncomingCommand::Uci);
        assert_eq!(stdout_rx.try_recv().err().unwrap(), TryRecvError::Empty);
    }
}
