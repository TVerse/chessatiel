use crate::position_hash_history::PositionHashHistory;
use crate::searcher::{Searcher, SearcherConfig};
use crate::statistics::StatisticsHolder;
use crate::time_manager::TimeManagerHandle;
use crate::transposition_table::TranspositionTable;
use crate::{EngineUpdate, SearchConfiguration};
use guts::Position;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::select;
use tokio::sync::watch;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;

#[derive(Debug)]
enum AggregatorMessage {
    StartSearch(
        mpsc::UnboundedSender<EngineUpdate>,
        oneshot::Receiver<()>,
        Position,
        PositionHashHistory,
        SearchConfiguration,
    ),
}

#[derive(Clone)]
pub struct AggregatorHandle {
    sender: mpsc::UnboundedSender<AggregatorMessage>,
}

impl AggregatorHandle {
    pub fn new(cancellation_rx: watch::Receiver<()>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = AggregatorActor::new(
            receiver,
            cancellation_rx,
            TimeManagerHandle::new(),
            Arc::new(Mutex::new(TranspositionTable::new())),
        );
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn start_search(
        &self,
        stop: oneshot::Receiver<()>,
        position: Position,
        position_history: PositionHashHistory,
        search_configuration: SearchConfiguration,
        updates: mpsc::UnboundedSender<EngineUpdate>,
    ) {
        let msg = AggregatorMessage::StartSearch(
            updates,
            stop,
            position,
            position_history,
            search_configuration,
        );

        let _ = self.sender.send(msg);
    }
}

struct AggregatorActor {
    receiver: mpsc::UnboundedReceiver<AggregatorMessage>,
    cancellation_rx: watch::Receiver<()>,
    time_manager: TimeManagerHandle,
    transposition_table: Arc<Mutex<TranspositionTable>>,
}

impl AggregatorActor {
    async fn handle_event(&mut self, message: AggregatorMessage) {
        debug!("Got aggregator message");
        match message {
            AggregatorMessage::StartSearch(updates, stop, position, position_history, config) => {
                info!("Starting search");
                let (result_tx, mut result_rx) = mpsc::unbounded_channel();
                let (stop_tx, stop_rx) = watch::channel(());
                let searcher_stop_rx = stop_rx.clone();
                self.time_manager.update(config.remaining_time).await;
                let searcher_config = SearcherConfig {
                    depth: config.depth,
                };
                let stats = Arc::new(StatisticsHolder::new());
                let stats_search = stats.clone();
                let mut stats_cancel_rx = self.cancellation_rx.clone();
                let mut stats_stop_rx = stop_rx.clone();
                let stats_updates_tx = updates.clone();
                let start = Instant::now();
                let _show_stats = tokio::task::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(5));
                    let mut previous_stats = stats.get_statistics();
                    loop {
                        select! {
                            _ = stats_stop_rx.changed() => break,
                            _ = stats_cancel_rx.changed() => break,
                            _ = interval.tick() => {
                                let new_stats = stats.get_statistics();
                                let nps =  (new_stats.nodes_searched - previous_stats.nodes_searched) / interval.period().as_secs();
                                let _ = stats_updates_tx.send(EngineUpdate::Info{
                                    nps: Some(nps),
                                    depth: Some(new_stats.current_depth),
                                    nodes: Some(new_stats.nodes_searched),
                                    tt_hits: Some(new_stats.tt_hits),
                                    score: None,
                                });
                                previous_stats = new_stats;
                            }
                        }
                    }
                    info!(
                        "Stats:\n{}\nTime taken: {:.3} seconds",
                        stats.get_statistics(),
                        Instant::now().duration_since(start).as_secs_f32()
                    )
                });
                let search_tt = self.transposition_table.clone();
                // Should end by itself after cancellation or dropping of the move receiver
                let _search_task = std::thread::spawn(move || {
                    let mut guard = search_tt.lock().unwrap();
                    let mut searcher = Searcher::new(
                        position_history,
                        position,
                        searcher_stop_rx,
                        searcher_config,
                        &stats_search,
                        &mut guard,
                    );
                    searcher.search(result_tx)
                });
                let timer = self.time_manager.start(stop_rx);

                let mut result = None;

                select! {
                    _ = async { while let Some(r) = result_rx.recv().await {
                        result = Some(r)
                    }} => {}
                    _ = timer => {}
                    _ = stop => {
                        let _ = stop_tx.send(());
                    }
                    _ = self.cancellation_rx.changed() => {
                        let _ = stop_tx.send(());
                    }
                }

                info!("Best move found: {:?}", result);
                if let Some(result) = result {
                    let _ = updates.send(EngineUpdate::Info {
                        nps: None,
                        depth: None,
                        nodes: None,
                        tt_hits: None,
                        score: Some(result.score.0),
                    });
                    let _ = updates.send(EngineUpdate::BestMove(result));
                }
            }
        }
    }

    fn new(
        receiver: mpsc::UnboundedReceiver<AggregatorMessage>,
        cancellation_rx: watch::Receiver<()>,
        time_manager: TimeManagerHandle,
        transposition_table: Arc<Mutex<TranspositionTable>>,
    ) -> Self {
        Self {
            receiver,
            cancellation_rx,
            time_manager,
            transposition_table,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await;
        }
    }
}
