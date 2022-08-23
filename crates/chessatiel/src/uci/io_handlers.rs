use crate::uci::protocol::{IncomingCommand, InfoPayload, OutgoingCommand, UciParser};
use std::io::BufRead;
use std::io::Write;

use log::warn;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct InputHandler<'a, I>
where
    I: BufRead,
{
    stdin: &'a mut I,
    tx: UnboundedSender<IncomingCommand>,
    tx_err: UnboundedSender<OutgoingCommand>,
    uci_parser: UciParser,
    buf: String,
}
impl<'a, I> InputHandler<'a, I>
where
    I: BufRead,
{
    pub fn new(
        stdin: &'a mut I,
        tx: UnboundedSender<IncomingCommand>,
        tx_err: UnboundedSender<OutgoingCommand>,
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
            let parsed = self.uci_parser.parse(self.buf.trim_end());
            match parsed {
                Ok(cmd) => self.tx.send(cmd).unwrap(),
                Err(err) => {
                    let error_text =
                        format!("Could not parse UCI input '{}': {err}", self.buf.trim_end());
                    warn!("{}", error_text);
                    self.tx_err
                        .send(OutgoingCommand::Info(InfoPayload {
                            string: Some(error_text),
                            ..InfoPayload::default()
                        }))
                        .unwrap();
                }
            }
        }
    }
}
pub struct OutputHandler<'a, O>
where
    O: Write,
{
    o: &'a mut O,
    rx: UnboundedReceiver<OutgoingCommand>,
}

impl<'a, O> OutputHandler<'a, O>
where
    O: Write,
{
    pub fn new(o: &'a mut O, rx: UnboundedReceiver<OutgoingCommand>) -> Self {
        Self { o, rx }
    }

    pub fn handle_one(&mut self) {
        let received = self.rx.blocking_recv().unwrap();
        writeln!(self.o, "{}", received).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::error::TryRecvError;

    #[test]
    fn parse_and_send_from_stdin() {
        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel();
        let (stdout_tx, mut stdout_rx) = mpsc::unbounded_channel();

        let stdin = "uci\n";

        let mut reader = BufReader::new(stdin.as_bytes());
        let mut input_handler = InputHandler::new(&mut reader, stdin_tx, stdout_tx);

        input_handler.handle_one();

        assert_eq!(stdin_rx.blocking_recv().unwrap(), IncomingCommand::Uci);
        assert_eq!(stdout_rx.try_recv().err().unwrap(), TryRecvError::Empty);
    }

    #[test]
    fn send_to_stdout() {
        let (stdout_tx, stdout_rx) = mpsc::unbounded_channel();

        let mut stdout = Vec::new();

        let mut output_handler = OutputHandler::new(&mut stdout, stdout_rx);

        stdout_tx
            .send(OutgoingCommand::Info(InfoPayload {
                string: Some("payload".to_owned()),
                ..InfoPayload::default()
            }))
            .unwrap();

        output_handler.handle_one();

        assert_eq!("info string payload \n", String::from_utf8(stdout).unwrap());
    }
}
