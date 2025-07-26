use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use webrtc::{
    api::{
        APIBuilder, interceptor_registry::register_default_interceptors, media_engine::MediaEngine,
    },
    ice_transport::{ice_candidate::RTCIceCandidate, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{RTCPeerConnection, configuration::RTCConfiguration},
};

use crate::client::{
    Client,
    error::Error,
    message::{Message, SignalMessage},
};

lazy_static! {
    static ref PEER_CONNECTION_MUTEX: Arc<Mutex<Option<Arc<RTCPeerConnection>>>> =
        Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidate>>> = Arc::new(Mutex::new(vec![]));
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

pub struct ConnectionManager {
    peer_connection: Arc<RTCPeerConnection>,
}

impl ConnectionManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
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

        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        // TODO: setup on_ice_candidate
        // let pc = Arc::downgrade(&peer_connection);
        // let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
        // let addr2 = answer_addr.clone();
        // peer_connection.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
        //     //println!("on_ice_candidate {:?}", c);

        //     let pc2 = pc.clone();
        //     let pending_candidates3 = Arc::clone(&pending_candidates2);
        //     let addr3 = addr2.clone();
        //     Box::pin(async move {
        //         if let Some(c) = c {
        //             if let Some(pc) = pc2.upgrade() {
        //                 let desc = pc.remote_description().await;
        //                 if desc.is_none() {
        //                     let mut cs = pending_candidates3.lock().await;
        //                     cs.push(c);
        //                 } else if let Err(err) = signal_candidate(&addr3, &c).await {
        //                     panic!("{}", err);
        //                 }
        //             }
        //         }
        //     })
        // }));

        Ok(Self {
            peer_connection: peer_connection,
        })
    }

    // TODO: create an init function if needed

    pub async fn create_offer(&self, room_client: &mut Client) -> Result<(), webrtc::Error> {
        todo!()
    }

    pub async fn receive_message(
        &mut self,
        message: &Message,
        room_client: &mut Client,
        is_creator: bool,
    ) -> Result<(), Error> {
        let signal_message = match message {
            // Message::Room { room_message } => {
            //     match room_message {
            //         RoomMessage::Join { username } => {
            //             // TODO: do something
            //         }
            //         _ => return Ok(())
            //     }
            // }
            Message::Signal { signal_message } => signal_message,
            _ => return Ok(()),
        };

        match signal_message {
            SignalMessage::Offer { payload } => todo!(),
            SignalMessage::Answer { payload } => todo!(),
            SignalMessage::Candidate { payload } => todo!(),
        }
    }
}
