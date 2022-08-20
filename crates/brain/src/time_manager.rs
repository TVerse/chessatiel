use crate::{ack, AckTx, RemainingTime};
use log::info;
use std::time::Duration;
use tokio::select;
use tokio::sync::{mpsc, watch};

#[derive(Debug)]
pub enum TimeManagerMessage {
    Update(AckTx, Option<RemainingTime>),
    Start(AckTx, watch::Receiver<()>),
}

pub struct TimeManagerHandle {
    sender: mpsc::UnboundedSender<TimeManagerMessage>,
}

impl TimeManagerHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = TimeManager::new(receiver);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn start(&self, stop: watch::Receiver<()>) {
        let (tx, rx) = ack();
        let msg = TimeManagerMessage::Start(tx, stop);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn update(&self, remaining_time: Option<RemainingTime>) {
        let (tx, rx) = ack();
        let msg = TimeManagerMessage::Update(tx, remaining_time);
        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

struct TimeManager {
    remaining_time: Option<RemainingTime>,
    receiver: mpsc::UnboundedReceiver<TimeManagerMessage>,
}

impl TimeManager {
    async fn handle_event(&mut self, msg: TimeManagerMessage) {
        match msg {
            TimeManagerMessage::Update(ack, remaining_time) => {
                self.remaining_time = remaining_time;
                let _ = ack.send(());
            }
            TimeManagerMessage::Start(ack, mut stop) => {
                let delay = self.delay_for_time();
                select! {
                    _ = delay => {
                        info!("Timer wakeup");
                        let _ = ack.send(());
                    }
                    _ = stop.changed() => {
                        info!("Timer got stop")
                    }
                }
            }
        }
    }

    async fn delay_for_time(&self) {
        let time = if let Some(remaining_time) = self.remaining_time {
            match remaining_time {
                RemainingTime::ForGame {
                    remaining,
                    increment,
                } => increment + ((remaining - increment) / 10),
                RemainingTime::ForMove(time) => time,
            }
        } else {
            Duration::from_secs(u64::MAX)
        };

        info!("Timer started for {time:?}");
        tokio::time::sleep(time).await;
        info!("Timer done");
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await
        }
    }

    pub fn new(receiver: mpsc::UnboundedReceiver<TimeManagerMessage>) -> Self {
        Self {
            remaining_time: None,
            receiver,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::watch;
    use tokio::time::Instant;

    #[tokio::test]
    async fn times_out_correctly() {
        let timer = TimeManagerHandle::new();
        timer
            .update(Some(RemainingTime::ForMove(Duration::from_secs(3))))
            .await;
        let (_tx, rx) = watch::channel(());
        let s = timer.start(rx);
        let upper_bound = tokio::time::sleep(Duration::from_secs(4));

        let now = Instant::now();
        select! {
            _ = s => {}
            _ = upper_bound => panic!("Upper bound should not stop first")
        };
        let after = Instant::now();
        let duration = after.duration_since(now);
        assert!(
            duration > Duration::from_secs(2),
            "Timer lower bound failed"
        );
    }
}
