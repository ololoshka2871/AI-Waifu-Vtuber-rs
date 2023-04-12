use serenity::{
    async_trait,
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};
use tracing::info;

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}
