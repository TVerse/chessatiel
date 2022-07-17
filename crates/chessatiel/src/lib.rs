mod brain;
pub mod lichess;

use tokio::sync::oneshot;
use tokio::sync::watch;

type AnswerRx<T> = oneshot::Receiver<T>;
type AnswerTx<T> = oneshot::Sender<T>;
type AckRx = AnswerRx<()>;
type AckTx = AnswerTx<()>;

fn ack() -> (AckTx, AckRx) {
    oneshot::channel()
}

fn answer<T>() -> (AnswerTx<T>, AnswerRx<T>) {
    oneshot::channel()
}

#[derive(Debug, Clone)]
struct Shutdown {
    shutdown: bool,
    notify: watch::Receiver<()>,
}

impl Shutdown {
    pub(crate) fn new(notify: watch::Receiver<()>) -> Self {
        Self {
            shutdown: false,
            notify,
        }
    }

    pub(crate) async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.notify.changed().await;

        self.shutdown = true;
    }
}
