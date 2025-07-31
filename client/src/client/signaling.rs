use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::Mutex;
use webrtc::{
    api::{
        APIBuilder, interceptor_registry::register_default_interceptors, media_engine::MediaEngine,
    },
    ice_transport::{
        ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
        ice_server::RTCIceServer,
    },
    interceptor::registry::Registry,
    peer_connection::{
        RTCPeerConnection, configuration::RTCConfiguration,
        sdp::session_description::RTCSessionDescription,
    },
};

use crate::client::{Client, SignalHandler, error::Error, message::SignalMessage};

lazy_static! {
    static ref PEER_CONNECTION_MUTEX: Arc<Mutex<Option<Arc<RTCPeerConnection>>>> =
        Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidate>>> = Arc::new(Mutex::new(vec![]));
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

async fn create_peer_connction() -> Result<RTCPeerConnection, webrtc::Error> {
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

pub async fn init_peer_connection(client: &Arc<Mutex<Client>>) -> Result<(), Error> {
    let peer_connection = match create_peer_connction().await {
        Ok(peer_connection) => peer_connection,
        Err(e) => return Err(Error::WebRTC { error: e }),
    };

    // TODO: add media tracks
    // peer_connection.add_track(todo!());

    let weak_client = Arc::downgrade(&client);
    peer_connection.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
        let inner_weak_client = weak_client.clone();
        Box::pin(async move {
            let candidate = match candidate {
                Some(candidate) => candidate,
                None => return,
            };

            let payload = match candidate.to_json() {
                Ok(payload) => payload,
                Err(_) => return,
            };

            let message = SignalMessage::Candidate { payload: payload };
            let json_string = match serde_json::to_string(&message) {
                Ok(json) => json,
                Err(_) => return,
            };

            let inner_self_ref = match inner_weak_client.upgrade() {
                Some(inner_self_ref) => inner_self_ref,
                None => return,
            };

            let mut client = inner_self_ref.lock().await;
            let _ = client.send_message(json_string).await;
        })
    }));

    client.lock().await.peer_connection = Some(Arc::new(peer_connection));

    Ok(())
}

impl SignalHandler for Client {
    async fn send_offer(&mut self) -> Result<(), Error> {
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
        let json_string = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json_string).await
    }

    async fn handle_offer(&mut self, offer: &RTCSessionDescription) -> Result<(), Error> {
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
        let json_string = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json_string).await
    }

    async fn handle_answer(&mut self, answer: &RTCSessionDescription) -> Result<(), Error> {
        let peer_connection = match &self.peer_connection {
            Some(peer_connection) => peer_connection,
            None => return Err(Error::PeerConnectionNotReady),
        };

        peer_connection
            .set_remote_description(answer.clone())
            .await
            .map_err(|e| Error::WebRTC { error: e })
    }

    async fn handle_candidate(&mut self, candidate: &RTCIceCandidateInit) -> Result<(), Error> {
        let peer_connection = match &self.peer_connection {
            Some(peer_connection) => peer_connection,
            None => return Err(Error::PeerConnectionNotReady),
        };

        peer_connection
            .add_ice_candidate(candidate.clone())
            .await
            .map_err(|e| Error::WebRTC { error: e })
    }
}
