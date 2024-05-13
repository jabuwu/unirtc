use std::sync::Arc;

use shadow_clone::shadow_clone;
use tokasm::sync::RwLock;
use tracing::{info, Level};
use unirtc as rtc;

#[tokasm::main]
async fn main() {
    unilog::init(Level::INFO, "webrtc_ice::agent::agent_internal=off,webrtc_ice::agent::agent_gather=off,webrtc::peer_connection=off,webrtc_ice::mdns=off,webrtc_mdns::conn=off");
    let configuration = rtc::Configuration {
        ice_servers: vec![rtc::IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let peer1 = Arc::new(RwLock::new(
        rtc::PeerConnection::new(&configuration).await.unwrap(),
    ));
    let peer2 = Arc::new(RwLock::new(
        rtc::PeerConnection::new(&configuration).await.unwrap(),
    ));

    let data_channel = Arc::new(RwLock::new(
        peer1
            .write()
            .await
            .create_data_channel("data", rtc::DataChannelInit::default())
            .await
            .unwrap(),
    ));
    setup_data_channel("peer1", data_channel).await;
    let offer = peer1.write().await.create_offer().await.unwrap();
    peer1
        .write()
        .await
        .set_local_description(&offer)
        .await
        .unwrap();
    peer2
        .write()
        .await
        .set_remote_description(&offer)
        .await
        .unwrap();
    let answer = peer2.write().await.create_answer().await.unwrap();
    peer2
        .write()
        .await
        .set_local_description(&answer)
        .await
        .unwrap();
    peer1
        .write()
        .await
        .set_remote_description(&answer)
        .await
        .unwrap();

    setup_exchange_ice_candidates(peer1.clone(), peer2.clone()).await;
    setup_exchange_ice_candidates(peer2.clone(), peer1.clone()).await;

    peer2
        .write()
        .await
        .on_data_channel(Box::new(|data_channel| {
            Box::pin(async move {
                setup_data_channel("peer2", Arc::new(RwLock::new(data_channel))).await;
            })
        }));

    tokasm::time::sleep_forever().await;
}

async fn setup_exchange_ice_candidates(
    peer: Arc<RwLock<rtc::PeerConnection>>,
    other_peer: Arc<RwLock<rtc::PeerConnection>>,
) {
    peer.write()
        .await
        .on_ice_candidate(Box::new(move |ice_candidate| {
            shadow_clone!(other_peer);
            Box::pin(async move {
                other_peer
                    .write()
                    .await
                    .add_ice_candidate(
                        ice_candidate.map(|ice_candidate| ice_candidate.to_init().unwrap()),
                    )
                    .await
                    .unwrap();
            })
        }));
}

async fn setup_data_channel(peer_name: &'static str, data_channel: Arc<RwLock<rtc::DataChannel>>) {
    let data_channel_inner = data_channel.clone();
    let data_channel = data_channel.write().await;
    {
        shadow_clone!(data_channel_inner);
        data_channel.on_open(Box::new(move || {
            shadow_clone!(peer_name, data_channel_inner);
            Box::pin(async move {
                info!("[{}] Sending message...", peer_name);
                data_channel_inner
                    .write()
                    .await
                    .send("Hello!".as_bytes())
                    .await
                    .unwrap();
            })
        }));
    }
    data_channel.on_message(Box::new(move |message, _| {
        Box::pin(async move {
            info!(
                "[{}] Received message: {}",
                peer_name,
                std::str::from_utf8(&message).unwrap()
            );
        })
    }));
}
