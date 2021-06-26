use std::io::Write;
use beak::OutgoingCommand;
use std::sync::mpsc::Receiver;

pub struct OutputHandler<'a, O>
    where O: Write {
    o: &'a mut O,
    rx: Receiver<OutgoingCommand>
}

impl<'a, O> OutputHandler<'a, O>
where O: Write {
    pub fn new(o: &'a mut O, rx: Receiver<OutgoingCommand>) -> Self {
        Self {
            o,
            rx,
        }
    }

    pub fn handle_one(&mut self) {
        let received = self.rx.recv().unwrap();
        writeln!(self.o, "{}", received).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use beak::InfoPayload;

    #[test]
    fn send_to_stdout() {
        let (stdout_tx, stdout_rx) = mpsc::channel();

        let mut stdout = Vec::new();

        let mut output_handler = OutputHandler::new(&mut stdout, stdout_rx);

        stdout_tx.send(OutgoingCommand::Info(InfoPayload::Nps(1234))).unwrap();

        output_handler.handle_one();

        assert_eq!("info nps 1234\n", String::from_utf8(stdout).unwrap());
    }
}