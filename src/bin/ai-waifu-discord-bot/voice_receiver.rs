use std::collections::HashMap;

use async_trait::async_trait;
use serenity::model::prelude::{ChannelId, GuildId};
use songbird::{
    model::{id::UserId, payload::Speaking},
    Event, EventContext, EventHandler as VoiceEventHandler,
};
use tokio::sync::mpsc::{Receiver, Sender};

use tracing::{error, trace};

#[derive(Debug, Clone)]
pub enum VoiceEvent {
    RegisterUser(UserId, u32),
    SpeakingStateUpdate {
        ssrc: u32,
        speaking: bool,
        guid: GuildId,
        ch_id: ChannelId,
    },
    VoicePacket(u32, Vec<i16>),
}

pub struct SpeakingStateUpdateListener(Sender<VoiceEvent>);

#[async_trait]
impl VoiceEventHandler for SpeakingStateUpdateListener {
    async fn act(&self, ev: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ev {
            Ctx::SpeakingStateUpdate(Speaking {
                speaking,
                ssrc,
                user_id,
                ..
            }) => {
                // Discord voice calls use RTP, where every sender uses a randomly allocated
                // *Synchronisation Source* (SSRC) to allow receivers to tell which audio
                // stream a received packet belongs to. As this number is not derived from
                // the sender's user_id, only Discord Voice Gateway messages like this one
                // inform us about which random SSRC a user has been allocated. Future voice
                // packets will contain *only* the SSRC.
                //
                // You can implement logic here so that you can differentiate users'
                // SSRCs and map the SSRC to the User ID and maintain this state.
                // Using this map, you can map the `ssrc` in `voice_packet`
                // to the user ID and handle their audio packets separately.
                trace!(
                    "Speaking state update: user {:?} has SSRC {:?}, using {:?}",
                    user_id, ssrc, speaking,
                );
                if let Some(user_id) = user_id {
                    if let Err(e) = self.0.send(VoiceEvent::RegisterUser(*user_id, *ssrc)).await {
                        error!("Failed to send voice event {:?}", e)
                    }
                }
            }
            _ => {
                // This listener only cares about speaking state updates.
                error!("Unexpected event: {:?}", ev);
                return None;
            }
        }

        None
    }
}
pub struct SpeakingUpdateListener {
    sender: Sender<VoiceEvent>,
    guid: GuildId,
    ch_id: ChannelId,
}

#[async_trait]
impl VoiceEventHandler for SpeakingUpdateListener {
    async fn act(&self, ev: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ev {
            Ctx::SpeakingUpdate(su_data) => {
                // You can implement logic here which reacts to a user starting
                // or stopping speaking, and to map their SSRC to User ID.
                if let Err(e) = self
                    .sender
                    .send(VoiceEvent::SpeakingStateUpdate {
                        ssrc: su_data.ssrc,
                        speaking: su_data.speaking,
                        guid: self.guid,
                        ch_id: self.ch_id,
                    })
                    .await
                {
                    error!("Failed to send voice event {:?}", e)
                }
            }
            _ => {
                // This listener only cares about speaking updates.
                error!("Unexpected event: {:?}", ev);
                return None;
            }
        }

        None
    }
}

pub struct VoicePacketListener(Sender<VoiceEvent>);

#[async_trait]
impl VoiceEventHandler for VoicePacketListener {
    async fn act(&self, ev: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ev {
            Ctx::VoicePacket(data) => {
                // You can implement logic here which reacts to a user's voice packet.
                // You can use the `ssrc` to map the packet to the user ID.
                if let Err(e) = self
                    .0
                    .send(VoiceEvent::VoicePacket(
                        data.packet.ssrc,
                        data.audio.clone().unwrap(),
                    ))
                    .await
                {
                    error!("Failed to send voice event {:?}", e)
                }
            }
            _ => {
                // This listener only cares about voice packets.
                error!("Unexpected event: {:?}", ev);
                return None;
            }
        }

        None
    }
}

pub fn create_voice_control_pair() -> (VoiceEventListenerBuilder, VoiceProcessor) {
    let (sender, receiver) = tokio::sync::mpsc::channel(16);
    (
        VoiceEventListenerBuilder { sender },
        VoiceProcessor::new(receiver),
    )
}

pub struct VoiceEventListenerBuilder {
    sender: Sender<VoiceEvent>,
}

impl VoiceEventListenerBuilder {
    pub fn build_state_update_listener(&self) -> SpeakingStateUpdateListener {
        SpeakingStateUpdateListener(self.sender.clone())
    }

    pub fn build_speaking_update_listener(
        &self,
        guid: GuildId,
        ch_id: ChannelId,
    ) -> SpeakingUpdateListener {
        SpeakingUpdateListener {
            sender: self.sender.clone(),
            guid,
            ch_id,
        }
    }

    pub fn build_voice_packet_listener(&self) -> VoicePacketListener {
        VoicePacketListener(self.sender.clone())
    }
}

pub struct VoiceProcessor {
    receiver: Receiver<VoiceEvent>,

    user_ssrc_map: HashMap<u32, UserId>,
    storage: HashMap<u32, Vec<i16>>,
}

impl VoiceProcessor {
    pub fn new(receiver: Receiver<VoiceEvent>) -> Self {
        Self {
            receiver,
            user_ssrc_map: HashMap::new(),
            storage: HashMap::new(),
        }
    }

    pub async fn try_get_user_voice(&mut self) -> Result<Option<(UserId, Vec<i16>, GuildId, ChannelId)>, ()> {
        if let Some(ev) = self.receiver.recv().await {
            match ev {
                VoiceEvent::RegisterUser(user_id, ssrc) => {
                    self.user_ssrc_map.insert(ssrc, user_id);
                }
                VoiceEvent::SpeakingStateUpdate { ssrc, speaking, guid, ch_id } => {
                    if speaking {
                        // start recording
                        self.storage.insert(ssrc, Vec::new());
                    } else {
                        // stop recording and return the recorded data
                        if let Some(res) = self.storage.remove(&ssrc) {
                            if res.len() > 4096 {
                                return Ok(Some((self.user_ssrc_map[&ssrc], res, guid, ch_id)));
                            }
                        }
                    }
                }
                VoiceEvent::VoicePacket(ssrc, data) => {
                    if let Some(storage) = self.storage.get_mut(&ssrc) {
                        storage.extend(data);
                    }
                }
            }
        }
        Ok(None)
    }
}
