use std::collections::HashMap;

use serenity::model::prelude::{ChannelId, GuildId};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum State {
    TextOnly,
    Voice,
}

pub struct VoiceChannelMap(HashMap<GuildId, HashMap<ChannelId, State>>);

impl VoiceChannelMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_voice_state(&mut self, guild_id: GuildId, channel_id: ChannelId) -> State {
        let state = self
            .0
            .entry(guild_id)
            .or_insert(HashMap::new())
            .entry(channel_id)
            .or_insert(State::TextOnly);

        *state
    }

    pub fn set_voice_state(&mut self, guild_id: GuildId, channel_id: ChannelId, state: State) {
        self.0
            .entry(guild_id)
            .or_insert(HashMap::new())
            .insert(channel_id, state);
    }
}

impl std::fmt::Debug for VoiceChannelMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}
