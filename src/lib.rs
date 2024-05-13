#![allow(missing_docs)]

use std::{collections::HashMap, future::Future, pin::Pin};

use maybe_sync::{dyn_maybe_send, dyn_maybe_send_sync};
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use webrtc::{
        data_channel::{data_channel_init::RTCDataChannelInit, RTCDataChannel},
        ice::candidate::{CandidatePairState, CandidateType},
        ice_transport::{
            ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
            ice_credential_type::RTCIceCredentialType,
            ice_server::RTCIceServer,
        },
        peer_connection::{
            configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
            policy::ice_transport_policy::RTCIceTransportPolicy,
            sdp::session_description::RTCSessionDescription, RTCPeerConnection,
        },
        stats::StatsReportType,
    };
}
#[cfg(target_arch = "wasm32")]
mod wasm {
    pub use js_sys::{Array, Object, Reflect, Uint8Array};
    pub use wasm_bindgen::{closure::Closure, JsValue};
    pub use wasm_bindgen_futures::{future_to_promise, JsFuture};
    pub use web_sys::{
        RtcConfiguration, RtcDataChannel, RtcDataChannelInit, RtcDataChannelType, RtcIceCandidate,
        RtcIceCandidateInit, RtcIceTransportPolicy, RtcPeerConnection, RtcPeerConnectionState,
        RtcSdpType, RtcSessionDescription, RtcSessionDescriptionInit, RtcStatsReport, TextEncoder,
    };
}

// TODO: remove the wasm unwraps in this file with `expect`

pub type OnOpenFn =
    Box<dyn_maybe_send_sync!((Fn() -> Pin<Box<dyn_maybe_send!(Future<Output = ()> + 'static)>>))>;
pub type OnCloseFn =
    Box<dyn_maybe_send_sync!((Fn() -> Pin<Box<dyn_maybe_send!(Future<Output = ()> + 'static)>>))>;
pub type OnMessageFn = Box<
    dyn_maybe_send_sync!(
        (Fn(Vec<u8>, bool) -> Pin<Box<dyn_maybe_send!(Future<Output = ()> + 'static)>>)
    ),
>;
pub type OnPeerConnectionStateChangeFn = Box<
    dyn_maybe_send_sync!(
        (Fn(PeerConnectionState) -> Pin<Box<dyn_maybe_send!(Future<Output = ()> + 'static)>>)
    ),
>;
pub type OnIceCandidateFn = Box<
    dyn_maybe_send_sync!(
        (Fn(Option<IceCandidate>) -> Pin<Box<dyn_maybe_send!(Future<Output = ()> + 'static)>>)
    ),
>;
pub type OnDataChannelFn = Box<
    dyn_maybe_send_sync!(
        (Fn(DataChannel) -> Pin<Box<dyn_maybe_send!(Future<Output = ()> + 'static)>>)
    ),
>;

#[cfg(not(target_arch = "wasm32"))]
async fn api<'a>() -> webrtc::api::API {
    use webrtc::{
        api::{
            interceptor_registry::register_default_interceptors, media_engine::MediaEngine,
            APIBuilder,
        },
        interceptor::registry::Registry,
    };

    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs().unwrap();
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine).unwrap();
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry);
    api.build()
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
    pub credential_type: IceCredentialType,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IceCredentialType {
    #[default]
    Unspecified,
    Password,
    Oauth,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IceTransportPolicy {
    #[default]
    All,
    Relay,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Configuration {
    pub ice_servers: Vec<IceServer>,
    pub ice_transport_policy: IceTransportPolicy,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Configuration> for native::RTCConfiguration {
    fn from(value: Configuration) -> Self {
        native::RTCConfiguration {
            ice_servers: value
                .ice_servers
                .into_iter()
                .map(|ice_server| native::RTCIceServer {
                    urls: ice_server.urls,
                    username: ice_server.username.unwrap_or(String::new()),
                    credential: ice_server.credential.unwrap_or(String::new()),
                    credential_type: match ice_server.credential_type {
                        IceCredentialType::Unspecified => native::RTCIceCredentialType::Unspecified,
                        IceCredentialType::Password => native::RTCIceCredentialType::Password,
                        IceCredentialType::Oauth => native::RTCIceCredentialType::Oauth,
                    },
                    ..Default::default()
                })
                .collect(),
            ice_transport_policy: match value.ice_transport_policy {
                IceTransportPolicy::All => native::RTCIceTransportPolicy::All,
                IceTransportPolicy::Relay => native::RTCIceTransportPolicy::Relay,
            },
            ..Default::default()
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Configuration> for wasm::RtcConfiguration {
    fn from(value: Configuration) -> Self {
        let mut configuration = wasm::RtcConfiguration::new();
        let ice_servers = wasm::Array::new();
        for IceServer {
            urls,
            username,
            credential,
            credential_type: _,
        } in value.ice_servers
        {
            let mut ice_server = wasm::Object::new();
            wasm::Reflect::set(
                &mut ice_server,
                &wasm::JsValue::from("urls"),
                &urls
                    .into_iter()
                    .fold(wasm::Array::new(), |array, url| {
                        array.push(&wasm::JsValue::from(url));
                        array
                    })
                    .into(),
            )
            .unwrap();
            if let Some(username) = username {
                wasm::Reflect::set(
                    &mut ice_server,
                    &wasm::JsValue::from("username"),
                    &wasm::JsValue::from(username),
                )
                .unwrap();
            }
            if let Some(credential) = credential {
                wasm::Reflect::set(
                    &mut ice_server,
                    &wasm::JsValue::from("credential"),
                    &wasm::JsValue::from(credential),
                )
                .unwrap();
            }
            ice_servers.push(&ice_server);
        }
        configuration.ice_servers(&ice_servers);
        configuration.ice_transport_policy(match value.ice_transport_policy {
            IceTransportPolicy::All => wasm::RtcIceTransportPolicy::All,
            IceTransportPolicy::Relay => wasm::RtcIceTransportPolicy::Relay,
        });
        configuration
    }
}

#[derive(Debug, Default, Clone)]
pub struct DataChannelInit {
    pub ordered: Option<bool>,
    pub max_retransmits: Option<u16>,
}

#[derive(Clone)]
pub struct DataChannel(
    #[cfg(not(target_arch = "wasm32"))] Arc<native::RTCDataChannel>,
    #[cfg(target_arch = "wasm32")] wasm::RtcDataChannel,
);

impl DataChannel {
    pub fn on_open(&self, handler: OnOpenFn) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.on_open(Box::new(move || {
                let future = handler();
                Box::pin(async move {
                    future.await;
                })
            }));
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let closure = wasm::Closure::wrap(Box::new(move || {
                let future = handler();
                _ = wasm::future_to_promise(async move {
                    future.await;
                    Ok(wasm::JsValue::UNDEFINED)
                });
            }) as Box<dyn Fn()>);
            self.0.set_onopen(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    pub fn on_close(&self, handler: OnCloseFn) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.on_close(Box::new(move || {
                let future = handler();
                Box::pin(async move {
                    future.await;
                })
            }));
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let closure = wasm::Closure::wrap(Box::new(move || {
                let future = handler();
                _ = wasm::future_to_promise(async move {
                    future.await;
                    Ok(wasm::JsValue::UNDEFINED)
                });
            }) as Box<dyn Fn()>);
            self.0.set_onclose(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    pub fn on_message(&self, handler: OnMessageFn) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.on_message(Box::new(move |message| {
                let is_string = message.is_string;
                let bytes = message.data.into_iter().collect::<Vec<_>>();
                let future = handler(bytes, is_string);
                Box::pin(async move {
                    future.await;
                })
            }));
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let closure = wasm::Closure::wrap(Box::new(move |event| {
                let data = wasm::Reflect::get(&event, &"data".into()).unwrap();
                let is_string = data.is_string();
                let bytes = if is_string {
                    let encoder = wasm::TextEncoder::new().unwrap();
                    encoder.encode_with_input(data.as_string().unwrap().as_str())
                } else {
                    wasm::Uint8Array::new(&data).to_vec()
                };
                let future = handler(bytes, is_string);
                _ = wasm::future_to_promise(async move {
                    future.await;
                    Ok(wasm::JsValue::UNDEFINED)
                });
            }) as Box<dyn Fn(wasm::JsValue)>);
            self.0.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    pub async fn send(&self, bytes: &[u8]) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = bytes.to_vec();
            self.0
                .send(&bytes.into())
                .await
                .map_err(|_| Error::FailedToSend)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.0.send_with_u8_array(bytes).unwrap();
            Ok(())
        }
    }

    pub async fn send_text(&self, str: &str) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0
                .send_text(str)
                .await
                .map_err(|_| Error::FailedToSend)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.0.send_with_str(str).unwrap();
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionDescription(
    #[cfg(not(target_arch = "wasm32"))] native::RTCSessionDescription,
    #[cfg(target_arch = "wasm32")] wasm::RtcSessionDescription,
);

impl SessionDescription {
    pub fn offer(sdp: &str) -> Result<Self, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(SessionDescription(
                native::RTCSessionDescription::offer(sdp.to_owned())
                    .map_err(|_| Error::FailedToCreateSessionDescription)?,
            ))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut init = wasm::RtcSessionDescriptionInit::new(wasm::RtcSdpType::Offer);
            init.sdp(sdp);
            let session_description =
                wasm::RtcSessionDescription::new_with_description_init_dict(&init)
                    .map_err(|_| Error::FailedToCreateSessionDescription)?;
            Ok(SessionDescription(session_description))
        }
    }

    pub fn answer(sdp: &str) -> Result<Self, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(SessionDescription(
                native::RTCSessionDescription::answer(sdp.to_owned())
                    .map_err(|_| Error::FailedToCreateSessionDescription)?,
            ))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut init = wasm::RtcSessionDescriptionInit::new(wasm::RtcSdpType::Answer);
            init.sdp(sdp);
            let session_description =
                wasm::RtcSessionDescription::new_with_description_init_dict(&init)
                    .map_err(|_| Error::FailedToCreateSessionDescription)?;
            Ok(SessionDescription(session_description))
        }
    }

    pub fn sdp(&self) -> String {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.sdp.clone()
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.0.sdp()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerConnectionState {
    Unspecified,
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<native::RTCPeerConnectionState> for PeerConnectionState {
    fn from(value: native::RTCPeerConnectionState) -> Self {
        match value {
            native::RTCPeerConnectionState::Unspecified => Self::Unspecified,
            native::RTCPeerConnectionState::New => Self::New,
            native::RTCPeerConnectionState::Connecting => Self::Connecting,
            native::RTCPeerConnectionState::Connected => Self::Connected,
            native::RTCPeerConnectionState::Disconnected => Self::Disconnected,
            native::RTCPeerConnectionState::Failed => Self::Failed,
            native::RTCPeerConnectionState::Closed => Self::Closed,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<wasm::RtcPeerConnectionState> for PeerConnectionState {
    fn from(value: wasm::RtcPeerConnectionState) -> Self {
        match value {
            wasm::RtcPeerConnectionState::New => Self::New,
            wasm::RtcPeerConnectionState::Connecting => Self::Connecting,
            wasm::RtcPeerConnectionState::Connected => Self::Connected,
            wasm::RtcPeerConnectionState::Disconnected => Self::Disconnected,
            wasm::RtcPeerConnectionState::Failed => Self::Failed,
            wasm::RtcPeerConnectionState::Closed => Self::Closed,
            _ => Self::Unspecified,
        }
    }
}

#[derive(Debug, Default)]
pub struct IceCandidateInit {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

#[derive(Debug)]
pub struct IceCandidate(
    #[cfg(not(target_arch = "wasm32"))] native::RTCIceCandidate,
    #[cfg(target_arch = "wasm32")] wasm::RtcIceCandidate,
);

impl IceCandidate {
    pub fn to_init(&self) -> Result<IceCandidateInit, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let init = self
                .0
                .to_json()
                .map_err(|_| Error::FailedToParseIceCandidate)?;
            Ok(IceCandidateInit {
                candidate: init.candidate,
                sdp_mid: init.sdp_mid,
                sdp_mline_index: init.sdp_mline_index,
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            let candidate = wasm::Reflect::get(&self.0, &"candidate".into())
                .unwrap()
                .as_string()
                .unwrap();
            let sdp_mid = if let Some(value) = wasm::Reflect::get(&self.0, &"sdpMid".into())
                .unwrap()
                .as_string()
            {
                Some(value)
            } else {
                None
            };
            let sdp_mline_index = if let Some(value) =
                wasm::Reflect::get(&self.0, &"sdpMLineIndex".into())
                    .unwrap()
                    .as_f64()
            {
                Some(value as u16)
            } else {
                None
            };
            Ok(IceCandidateInit {
                candidate,
                sdp_mid,
                sdp_mline_index,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatsReportType {
    CandidatePair(CandidatePairStats),
    LocalCandidate(CandidateStats),
    RemoteCandidate(CandidateStats),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidatePairState {
    Unspecified,
    Waiting,
    InProgress,
    Failed,
    Succeeded,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<native::CandidatePairState> for CandidatePairState {
    fn from(value: native::CandidatePairState) -> Self {
        match value {
            native::CandidatePairState::Unspecified => Self::Unspecified,
            native::CandidatePairState::Waiting => Self::Waiting,
            native::CandidatePairState::InProgress => Self::InProgress,
            native::CandidatePairState::Failed => Self::Failed,
            native::CandidatePairState::Succeeded => Self::Succeeded,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<String> for CandidatePairState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "waiting" => CandidatePairState::Waiting,
            "in-progress" => CandidatePairState::InProgress,
            "failed" => CandidatePairState::Failed,
            "succeeded" => CandidatePairState::Succeeded,
            _ => CandidatePairState::Unspecified,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CandidatePairStats {
    pub id: String,
    pub local_candidate_id: String,
    pub remote_candidate_id: String,
    pub state: CandidatePairState,
    pub nominated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateType {
    Unspecified,
    Host,
    ServerReflexive,
    PeerReflexive,
    Relay,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<native::CandidateType> for CandidateType {
    fn from(value: native::CandidateType) -> Self {
        match value {
            native::CandidateType::Unspecified => Self::Unspecified,
            native::CandidateType::Host => Self::Host,
            native::CandidateType::ServerReflexive => Self::ServerReflexive,
            native::CandidateType::PeerReflexive => Self::PeerReflexive,
            native::CandidateType::Relay => Self::Relay,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<String> for CandidateType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "host" => CandidateType::Host,
            "srflx" => CandidateType::ServerReflexive,
            "prflx" => CandidateType::PeerReflexive,
            "relay" => CandidateType::Relay,
            _ => CandidateType::Unspecified,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CandidateStats {
    pub id: String,
    pub candidate_type: CandidateType,
}

#[derive(Debug)]
pub struct PeerConnection(
    #[cfg(not(target_arch = "wasm32"))] native::RTCPeerConnection,
    #[cfg(target_arch = "wasm32")] wasm::RtcPeerConnection,
);

impl PeerConnection {
    pub async fn new(configuration: &Configuration) -> Result<Self, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let api = api().await;
            let configuration = native::RTCConfiguration::from(configuration.clone());
            let peer = api
                .new_peer_connection(configuration)
                .await
                .map_err(|_| Error::FailedToCreatePeer)?;
            Ok(PeerConnection(peer))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let configuration = wasm::RtcConfiguration::from(configuration.clone());
            Ok(PeerConnection(
                wasm::RtcPeerConnection::new_with_configuration(&configuration)
                    .map_err(|_| Error::FailedToCreatePeer)?,
            ))
        }
    }

    pub async fn create_offer(&self) -> Result<SessionDescription, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(SessionDescription(
                self.0
                    .create_offer(None)
                    .await
                    .map_err(|_| Error::FailedToCreateOffer)?,
            ))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Ok(SessionDescription(wasm::RtcSessionDescription::from(
                wasm::JsFuture::from(self.0.create_offer())
                    .await
                    .map_err(|_| Error::FailedToCreateOffer)?,
            )))
        }
    }

    pub async fn create_answer(&self) -> Result<SessionDescription, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(SessionDescription(
                self.0
                    .create_answer(None)
                    .await
                    .map_err(|_| Error::FailedToCreateAnswer)?,
            ))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Ok(SessionDescription(wasm::RtcSessionDescription::from(
                wasm::JsFuture::from(self.0.create_answer())
                    .await
                    .map_err(|_| Error::FailedToCreateAnswer)?,
            )))
        }
    }

    pub async fn set_local_description(
        &self,
        session_description: &SessionDescription,
    ) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0
                .set_local_description(session_description.0.clone())
                .await
                .map_err(|_| Error::FailedToSetLocalDescription)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut init = wasm::RtcSessionDescriptionInit::new(session_description.0.type_());
            init.sdp(&session_description.sdp());
            wasm::JsFuture::from(self.0.set_local_description(&init))
                .await
                .map_err(|_| Error::FailedToSetLocalDescription)?;
            Ok(())
        }
    }

    pub async fn set_remote_description(
        &self,
        session_description: &SessionDescription,
    ) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0
                .set_remote_description(session_description.0.clone())
                .await
                .map_err(|_| Error::FailedToSetRemoteDescription)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut init = wasm::RtcSessionDescriptionInit::new(session_description.0.type_());
            init.sdp(&session_description.sdp());
            wasm::JsFuture::from(self.0.set_remote_description(&init))
                .await
                .map_err(|_| Error::FailedToSetRemoteDescription)?;
            Ok(())
        }
    }

    pub fn on_connection_state_change(&self, handler: OnPeerConnectionStateChangeFn) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0
                .on_peer_connection_state_change(Box::new(move |state| {
                    let future = handler(PeerConnectionState::from(state));
                    Box::pin(async move {
                        future.await;
                    })
                }));
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let peer = self.0.clone();
            let closure = wasm::Closure::wrap(Box::new(move |_event: wasm::JsValue| {
                let future = handler(PeerConnectionState::from(peer.connection_state()));
                _ = wasm::future_to_promise(async move {
                    future.await;
                    Ok(wasm::JsValue::UNDEFINED)
                });
            }) as Box<dyn Fn(wasm::JsValue)>);
            self.0
                .set_onconnectionstatechange(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    pub fn on_ice_candidate(&self, handler: OnIceCandidateFn) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.on_ice_candidate(Box::new(move |ice_candidate| {
                let future =
                    handler(ice_candidate.map(|ice_candidate| IceCandidate(ice_candidate)));
                Box::pin(async move {
                    future.await;
                })
            }));
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let closure = wasm::Closure::wrap(Box::new(move |ice_candidate: wasm::JsValue| {
                let candidate = wasm::Reflect::get(&ice_candidate, &"candidate".into()).unwrap();
                let future = if candidate.is_object() {
                    handler(Some(IceCandidate(wasm::RtcIceCandidate::from(candidate))))
                } else {
                    handler(None)
                };
                _ = wasm::future_to_promise(async move {
                    future.await;
                    Ok(wasm::JsValue::UNDEFINED)
                });
            }) as Box<dyn Fn(wasm::JsValue)>);
            self.0
                .set_onicecandidate(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    pub async fn add_ice_candidate(
        &self,
        ice_candidate: Option<IceCandidateInit>,
    ) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(ice_candidate) = ice_candidate {
                self.0
                    .add_ice_candidate(native::RTCIceCandidateInit {
                        candidate: ice_candidate.candidate,
                        sdp_mid: ice_candidate.sdp_mid,
                        sdp_mline_index: ice_candidate.sdp_mline_index,
                        ..Default::default()
                    })
                    .await
                    .map_err(|_| Error::FailedToAddIceCandidate)?;
            }
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ice_candidate) = ice_candidate {
                let mut ice_candidate_init =
                    wasm::RtcIceCandidateInit::new(&ice_candidate.candidate);
                ice_candidate_init
                    .sdp_m_line_index(Some(ice_candidate.sdp_mline_index.unwrap_or(0)));
                ice_candidate_init.sdp_mid(Some(
                    ice_candidate
                        .sdp_mid
                        .as_ref()
                        .map(|str| str.as_str())
                        .unwrap_or("0"),
                ));
                wasm::JsFuture::from(
                    self.0
                        .add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(
                            &ice_candidate_init,
                        )),
                )
                .await
                .map_err(|_| Error::FailedToAddIceCandidate)?;
            } else {
                wasm::JsFuture::from(
                    self.0
                        .add_ice_candidate_with_opt_rtc_ice_candidate_init(None),
                )
                .await
                .map_err(|_| Error::FailedToAddIceCandidate)?;
            }
            Ok(())
        }
    }

    pub async fn create_data_channel(
        &self,
        label: &str,
        options: DataChannelInit,
    ) -> Result<DataChannel, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(DataChannel(
                self.0
                    .create_data_channel(
                        label,
                        Some(native::RTCDataChannelInit {
                            ordered: options.ordered,
                            max_retransmits: options.max_retransmits,
                            ..Default::default()
                        }),
                    )
                    .await
                    .map_err(|_| Error::FailedToCreateDataChannel)?,
            ))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut data_channel_init = wasm::RtcDataChannelInit::new();
            if let Some(ordered) = options.ordered {
                data_channel_init.ordered(ordered);
            }
            if let Some(max_retransmits) = options.max_retransmits {
                data_channel_init.max_retransmits(max_retransmits);
            }
            let data_channel = self.0.create_data_channel(label);
            data_channel.set_binary_type(wasm::RtcDataChannelType::Arraybuffer);
            Ok(DataChannel(data_channel))
        }
    }

    pub fn on_data_channel(&self, handler: OnDataChannelFn) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.on_data_channel(Box::new(move |data_channel| {
                let future = handler(DataChannel(data_channel));
                Box::pin(async move {
                    future.await;
                })
            }));
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let closure = wasm::Closure::wrap(Box::new(move |event: wasm::JsValue| {
                let channel = js_sys::Reflect::get(&event, &"channel".into()).unwrap();
                let data_channel = wasm::RtcDataChannel::from(channel);
                data_channel.set_binary_type(wasm::RtcDataChannelType::Arraybuffer);
                let future = handler(DataChannel(data_channel));
                _ = wasm::future_to_promise(async move {
                    future.await;
                    Ok(wasm::JsValue::UNDEFINED)
                });
            }) as Box<dyn Fn(wasm::JsValue)>);
            self.0
                .set_ondatachannel(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    pub async fn close(&self) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.close().await.map_err(|_| Error::FailedToClose)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.0.close();
            Ok(())
        }
    }

    pub async fn stats(&self) -> Result<HashMap<String, StatsReportType>, Error> {
        let mut reports = HashMap::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            for (id, report) in self.0.get_stats().await.reports {
                match report {
                    native::StatsReportType::CandidatePair(stats) => {
                        reports.insert(
                            id,
                            StatsReportType::CandidatePair(CandidatePairStats {
                                id: stats.id,
                                local_candidate_id: stats.local_candidate_id,
                                remote_candidate_id: stats.remote_candidate_id,
                                state: CandidatePairState::from(stats.state),
                                nominated: stats.nominated,
                            }),
                        );
                    }
                    native::StatsReportType::LocalCandidate(stats) => {
                        reports.insert(
                            id,
                            StatsReportType::LocalCandidate(CandidateStats {
                                id: stats.id,
                                candidate_type: CandidateType::from(stats.candidate_type),
                            }),
                        );
                    }
                    native::StatsReportType::RemoteCandidate(stats) => {
                        reports.insert(
                            id,
                            StatsReportType::RemoteCandidate(CandidateStats {
                                id: stats.id,
                                candidate_type: CandidateType::from(stats.candidate_type),
                            }),
                        );
                    }
                    _ => {}
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            fn get_string(value: &wasm::JsValue, key: &str) -> Option<String> {
                wasm::Reflect::get(value, &key.into())
                    .ok()
                    .and_then(|value| value.as_string())
            }
            fn get_bool(value: &wasm::JsValue, key: &str) -> Option<bool> {
                wasm::Reflect::get(value, &key.into())
                    .ok()
                    .and_then(|value| value.as_bool())
            }
            let stats = wasm::RtcStatsReport::from(
                wasm::JsFuture::from(self.0.get_stats())
                    .await
                    .map_err(|_| Error::FailedToGetStats)?,
            );
            for result in stats.entries() {
                let report = wasm::Array::from(&result.map_err(|_| Error::FailedToGetStats)?);
                let Some(id) = report.get(0).as_string() else {
                    continue;
                };
                let stats = report.get(1);
                let Some(ty) = get_string(&stats, "type") else {
                    continue;
                };
                match ty.as_str() {
                    "candidate-pair" => {
                        let Some(local_candidate_id) = get_string(&stats, "localCandidateId")
                        else {
                            continue;
                        };
                        let Some(remote_candidate_id) = get_string(&stats, "remoteCandidateId")
                        else {
                            continue;
                        };
                        let Some(state) = get_string(&stats, "state") else {
                            continue;
                        };
                        let Some(nominated) = get_bool(&stats, "nominated") else {
                            continue;
                        };
                        reports.insert(
                            id.clone(),
                            StatsReportType::CandidatePair(CandidatePairStats {
                                id,
                                local_candidate_id,
                                remote_candidate_id,
                                state: CandidatePairState::from(state),
                                nominated,
                            }),
                        );
                    }
                    "local-candidate" => {
                        let Some(candidate_type) = get_string(&stats, "candidateType") else {
                            continue;
                        };
                        reports.insert(
                            id.clone(),
                            StatsReportType::LocalCandidate(CandidateStats {
                                id,
                                candidate_type: CandidateType::from(candidate_type),
                            }),
                        );
                    }
                    "remote-candidate" => {
                        let Some(candidate_type) = get_string(&stats, "candidateType") else {
                            continue;
                        };
                        reports.insert(
                            id.clone(),
                            StatsReportType::RemoteCandidate(CandidateStats {
                                id,
                                candidate_type: CandidateType::from(candidate_type),
                            }),
                        );
                    }
                    _ => {}
                }
            }
        }
        Ok(reports)
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Failed to create peer.
    #[error("Failed to create peer.")]
    FailedToCreatePeer,
    /// Failed to create offer.
    #[error("Failed to create offer.")]
    FailedToCreateOffer,
    /// Failed to create answer.
    #[error("Failed to create answer.")]
    FailedToCreateAnswer,
    /// Failed to set local description.
    #[error("Failed to set local description.")]
    FailedToSetLocalDescription,
    /// Failed to set remote description.
    #[error("Failed to set remote description.")]
    FailedToSetRemoteDescription,
    /// Failed to add ice candidate.
    #[error("Failed to add ice candidate.")]
    FailedToAddIceCandidate,
    /// Failed to create data channel.
    #[error("Failed to create data channel.")]
    FailedToCreateDataChannel,
    /// Failed to send.
    #[error("Failed to send.")]
    FailedToSend,
    /// Failed to create session description.
    #[error("Failed to create session description.")]
    FailedToCreateSessionDescription,
    /// Failed to parse ICE candidate.
    #[error("Failed to parse ICE candidate.")]
    FailedToParseIceCandidate,
    /// Failed to close.
    #[error("Failed to close.")]
    FailedToClose,
    /// Failed to get stats.
    #[error("Failed to get stats.")]
    FailedToGetStats,
}
