mod brain;
pub mod lichess;

use tokio::sync::oneshot;

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
