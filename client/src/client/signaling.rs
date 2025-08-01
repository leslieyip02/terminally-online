use std::{fs::File, io::BufReader, sync::Arc, time::Duration};

use lazy_static::lazy_static;
use tokio::{io::AsyncWriteExt, sync::Mutex};
use tracing::info;
use webrtc::{
    api::{
        APIBuilder,
        interceptor_registry::register_default_interceptors,
        media_engine::{MIME_TYPE_H264, MediaEngine},
    },
    ice_transport::{
        ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
        ice_server::RTCIceServer,
    },
    interceptor::registry::Registry,
    media::{Sample, io::h264_reader::H264Reader},
    peer_connection::{
        RTCPeerConnection, configuration::RTCConfiguration,
        sdp::session_description::RTCSessionDescription,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::{
        TrackLocal, TrackLocalWriter, track_local_static_rtp::TrackLocalStaticRTP,
        track_local_static_sample::TrackLocalStaticSample,
    },
};

use crate::{
    chat::Chatbox,
    client::{Client, SignalHandler, error::Error, message::SignalMessage},
};

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

    // Set up on_track handler
    peer_connection.on_track(Box::new(move |track, _, _| {
        Box::pin(async move {
            info!(
                "Received remote track: {} (codec {})",
                track.kind(),
                track.codec().capability.mime_type
            );

            // Create a local track to forward RTP to other peers
            let local_track = Arc::new(TrackLocalStaticRTP::new(
                track.codec().capability.clone(),
                String::from("video"),
                String::from("webrtc-rs"),
            ));

            tokio::spawn(async move {
                info!("Creating file");
                let mut file = match tokio::fs::File::create("received.h264").await {
                    Ok(f) => f,
                    Err(e) => {
                        info!("Could not create output file: {e}");
                        return;
                    }
                };
                info!("Created file");

                while let Ok((rtp, _)) = track.read_rtp().await {
                    info!("Reading RTP");
                    // Write to rebroadcast track
                    let _ = local_track.write_rtp(&rtp).await;

                    // Write to disk
                    if !rtp.payload.is_empty() {
                        let _ = file.write_all(&[0x00, 0x00, 0x00, 0x01]).await;
                        let _ = file.write_all(&rtp.payload).await;
                    }
                }
            });
        })
    }));

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

pub async fn send_video(client: &Arc<Mutex<Client>>) -> Result<(), Error> {
    let client = client.lock().await;
    let peer_connection = match &client.peer_connection {
        Some(peer_connection) => peer_connection,
        None => return Err(Error::PeerConnectionNotReady),
    };
    let video_track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: String::from(MIME_TYPE_H264),
            ..Default::default()
        },
        String::from("video"),
        String::from("webrtc-rs"),
    ));

    let rtp_sender = peer_connection
        .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await
        .map_err(|e| Error::WebRTC { error: e })?;

    // Read incoming RTCP packets
    // Before these packets are returned they are processed by interceptors. For things
    // like NACK this needs to be called.
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
    });

    info!("starting video thread");
    tokio::spawn(async move {
        info!("Opening file");
        let file = match File::open("tmp.h264") {
            Ok(file) => file,
            Err(_) => return,
        };
        info!("Opened file");

        let reader = BufReader::new(file);
        let mut h264 = H264Reader::new(reader, 1_048_576);

        let mut ticker = tokio::time::interval(Duration::from_millis(33));
        loop {
            info!("Sending sample");
            let nal = match h264.next_nal() {
                Ok(nal) => nal,
                Err(err) => {
                    info!("Stopped sending samples: {err}");
                    break;
                }
            };

            let sample = Sample {
                data: nal.data.freeze(),
                duration: Duration::from_secs(1),
                ..Default::default()
            };
            match video_track.write_sample(&sample).await {
                Ok(_) => {}
                Err(_) => break,
            }
            info!("Sent sample");

            let _ = ticker.tick().await;
        }
    });
    info!("started video thread");

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
