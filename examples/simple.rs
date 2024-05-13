use std::sync::Arc;

use shadow_clone::shadow_clone;
use tokasm::sync::RwLock;
use tracing::{error, info, Level};
use unirtc as rtc;

// macro just to make the example code a bit easier to read
macro_rules! w {
    ($ident:ident) => {
        $ident.write().await
    };
}

#[tokasm::main]
async fn main() {
    unilog::init(Level::INFO, "webrtc_ice::agent::agent_internal=off,webrtc_ice::agent::agent_gather=off,webrtc::peer_connection=off,webrtc_ice::mdns=off,webrtc_mdns::conn=off");
    peer_to_peer().await.unwrap();
    tokasm::time::sleep_forever().await;
}

async fn peer_to_peer() -> Result<(), rtc::Error> {
    let configuration = rtc::Configuration {
        ice_servers: vec![rtc::IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let peer1 = Arc::new(RwLock::new(
        rtc::PeerConnection::new(&configuration).await?,
    ));
    let peer2 = Arc::new(RwLock::new(
        rtc::PeerConnection::new(&configuration).await?,
    ));

    let data_channel = Arc::new(RwLock::new(
        w!(peer1)
            .create_data_channel("data", rtc::DataChannelInit::default())
            .await?,
    ));
    setup_data_channel("peer1", data_channel).await;
    let offer = w!(peer1).create_offer().await?;
    w!(peer1).set_local_description(&offer).await?;
    w!(peer2).set_remote_description(&offer).await?;
    let answer = w!(peer2).create_answer().await?;
    w!(peer2).set_local_description(&answer).await?;
    w!(peer1).set_remote_description(&answer).await?;

    setup_exchange_ice_candidates("peer1", peer1.clone(), peer2.clone()).await;
    setup_exchange_ice_candidates("peer2", peer2.clone(), peer1.clone()).await;

    w!(peer2).on_data_channel(Box::new(|data_channel| {
        Box::pin(async move {
            setup_data_channel("peer2", Arc::new(RwLock::new(data_channel))).await;
        })
    }));
    Ok(())
}

async fn setup_exchange_ice_candidates(
peer_name: &'static str,
    peer: Arc<RwLock<rtc::PeerConnection>>,
    other_peer: Arc<RwLock<rtc::PeerConnection>>,
) {
    w!(peer).on_ice_candidate(Box::new(move |ice_candidate| {
        shadow_clone!(other_peer);
        Box::pin(async move {
            match ice_candidate.map(|ice_candidate| ice_candidate.to_init()).transpose() {
                Ok(ice_candidate_init) => {
                    if let Err(err) = w!(other_peer)
                        .add_ice_candidate(
                            ice_candidate_init,
                        )
                        .await {
                            error!("[{}] Failed to add ice candidate: {:?}", peer_name, err);
                        }
                }
                Err(err) => {
                    error!("[{}] Failed to parse ice candidate: {:?}", peer_name, err);
                }
            }
        })
    }));
}

async fn setup_data_channel(peer_name: &'static str, data_channel: Arc<RwLock<rtc::DataChannel>>) {
    let data_channel_inner = data_channel.clone();
    let data_channel = w!(data_channel);
    {
        shadow_clone!(data_channel_inner);
        data_channel.on_open(Box::new(move || {
            shadow_clone!(peer_name, data_channel_inner);
            Box::pin(async move {
                info!("[{}] Sending message...", peer_name);
                if let Err(err) = w!(data_channel_inner).send("Hello!".as_bytes()).await {
                    error!("[{}] Failed to send message: {:?}", peer_name, err);
                }
            })
        }));
    }
    data_channel.on_message(Box::new(move |message, _| {
        Box::pin(async move {
            let message = match std::str::from_utf8(&message) {
                Ok(utf8) => utf8,
                Err(_) => "[binary data]"
            };
            info!(
                "[{}] Received message: {}",
                peer_name,
                message,
            );
        })
    }));
}
