use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::Mutex;
use webrtc::{
    api::{
        APIBuilder, interceptor_registry::register_default_interceptors, media_engine::MediaEngine,
    },
    ice_transport::{ice_candidate::RTCIceCandidate, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{
        RTCPeerConnection, configuration::RTCConfiguration,
        sdp::session_description::RTCSessionDescription,
    },
};

use crate::client::{error::Error, message::SignalMessage, Client, SignalHandler};

lazy_static! {
    static ref PEER_CONNECTION_MUTEX: Arc<Mutex<Option<Arc<RTCPeerConnection>>>> =
        Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidate>>> = Arc::new(Mutex::new(vec![]));
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

pub(crate) async fn create_peer_connction() -> Result<RTCPeerConnection, webrtc::Error> {
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut engine = MediaEngine::default();
    engine.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut engine)?;

    let api = APIBuilder::new()
        .with_media_engine(engine)
        .with_interceptor_registry(registry)
        .build();

    api.new_peer_connection(config).await
}

impl SignalHandler for Client {
    async fn send_offer_message(&mut self) -> Result<(), Error> {
        let peer_connection = match &self.peer_connection {
            Some(peer_connection) => peer_connection,
            None => return Err(Error::PeerConnectionNotReady),
        };

        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| Error::WebRTC { error: e })?;
        peer_connection
            .set_local_description(offer.clone())
            .await
            .map_err(|e| Error::WebRTC { error: e })?;

        let message = SignalMessage::Offer {
            payload: offer.clone(),
        };
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json).await
    }

    async fn handle_offer_message(&mut self, offer: &RTCSessionDescription) -> Result<(), Error> {
        let peer_connection = match &self.peer_connection {
            Some(peer_connection) => peer_connection,
            None => return Err(Error::PeerConnectionNotReady),
        };

        peer_connection
            .set_remote_description(offer.clone())
            .await
            .map_err(|e| Error::WebRTC { error: e })?;

        let answer = peer_connection
            .create_answer(None)
            .await
            .map_err(|e| Error::WebRTC { error: e })?;

        peer_connection
            .set_local_description(answer.clone())
            .await
            .map_err(|e| Error::WebRTC { error: e })?;

        let message = SignalMessage::Answer {
            payload: answer.clone(),
        };
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json).await
    }

    async fn handle_answer_message() -> Result<(), Error> {
        Ok(())
    }
}
