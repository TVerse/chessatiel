use crate::lichess::game::{GameClient, GameStateEvent};
use crate::lichess::LichessClient;
use anyhow::Result;
use futures::prelude::stream::*;
use log::*;
use tokio::select;
use tokio::sync::oneshot;

pub struct Engine {
    game_client: GameClient,
    game_id: String,
}

impl Engine {
    pub fn new(lichess_client: LichessClient, game_id: String) -> Self {
        Self {
            game_client: GameClient::new(lichess_client, game_id.clone()),
            game_id,
        }
    }

    pub async fn run(self) -> oneshot::Sender<()> {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            // TODO game end also closes the connection, is this select necessary?
            let handle_events = self.handle_events();
            select! {
                _res = handle_events => { info!("Done handling game {} (by handler)", self.game_id) }
                _res = rx => { info!("Done handling game {} (by channel)", self.game_id) }
            }
        });
        tx
    }

    pub async fn handle_events(&self) -> Result<()> {
        self.game_client
            .get_game_events()
            .await?
            .for_each(|e| async {
                match self.handle_event(e).await {
                    Ok(()) => (),
                    Err(e) => error!("Got an error handling game event: {}", e),
                }
            })
            .await;
        Ok(())
    }

    async fn handle_event(&self, event: GameStateEvent) -> Result<()> {
        debug!("Handling game event {:?}", event);
        match event {
            GameStateEvent::GameFull { .. } => Ok(()),
            GameStateEvent::GameState { .. } => Ok(()),
            GameStateEvent::ChatLine => {
                info!("Got a chat message");
                Ok(())
            }
        }
    }
}
