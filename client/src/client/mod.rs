use std::{sync::Arc, time::Duration};

use futures::{SinkExt, StreamExt};
use tokio::{sync::mpsc::Receiver, time::timeout};
use tracing::info;
use webrtc::{
    api::media_engine::MIME_TYPE_H264,
    ice_transport::ice_candidate::RTCIceCandidateInit,
    media::Sample,
    peer_connection::{RTCPeerConnection, sdp::session_description::RTCSessionDescription},
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::{TrackLocal, track_local_static_sample::TrackLocalStaticSample},
};

use crate::{
    chat::command::{ChatboxCommand, ChatboxInput},
    client::{
        error::Error,
        message::{Message, SignalMessage},
        room::{MessageReceiver, WriteStream},
    },
    video::webcam::Webcam,
};

pub mod error;
pub mod message;
pub mod room;
pub mod signaling;

pub struct Client {
    username: String,
    http_client: reqwest::Client,
    write_stream: Option<WriteStream>,
    message_receiver: Option<Receiver<Result<Message, Error>>>,
    peer_connection: Option<Arc<RTCPeerConnection>>,
}

impl Client {
    const TIMEOUT: Duration = Duration::from_millis(3000);

    pub fn new() -> Self {
        let username = whoami::username();

        Self {
            username: username,
            http_client: reqwest::Client::new(),
            write_stream: None,
            message_receiver: None,
            peer_connection: None,
        }
    }

    async fn send_message(&mut self, json_string: String) -> Result<(), Error> {
        let stream = match &mut self.write_stream {
            Some(stream) => stream,
            None => return Err(Error::NotConnected),
        };

        let response = match timeout(Self::TIMEOUT, stream.send(json_string.into())).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        match response {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::SendMessage),
        }
    }

    pub async fn receive_input(&mut self, input: &ChatboxInput) -> Result<Option<String>, Error> {
        match input {
            ChatboxInput::Message(content) => {
                self.send_chat(content).await?;
                Ok(None)
            }
            ChatboxInput::Command(command) => match command {
                ChatboxCommand::Create => {
                    let room_id = self.create_and_connect_to_room().await?;
                    Ok(format!("room id = {}", room_id).into())
                }
                ChatboxCommand::Join { room_id } => {
                    self.join_and_connect_to_room(room_id).await?;
                    Ok(None)
                }
                ChatboxCommand::Stream => {
                    self.start_stream().await?;
                    self.send_offer().await?;
                    Ok(None)
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    pub async fn receive_message(&mut self, message: &Message) -> Result<(), Error> {
        match message {
            Message::Room { .. } => Ok(()),
            Message::Signal { signal_message } => match signal_message {
                SignalMessage::Offer { payload } => self.handle_offer(&payload).await,
                SignalMessage::Answer { payload } => self.handle_answer(&payload).await,
                SignalMessage::Candidate { payload } => self.handle_candidate(&payload).await,
            },
        }
    }

    pub async fn poll_message(&mut self) -> Option<Result<Message, Error>> {
        match self.message_receiver.as_mut() {
            Some(rx) => rx.recv().await,
            None => None,
        }
    }

    pub async fn start_stream(&mut self) -> Result<(), Error> {
        let peer_connection = match &self.peer_connection {
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

        tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        });

        info!("starting video thread");
        tokio::spawn(async move {
            let mut webcam = Webcam::new();
            webcam.start();

            loop {
                match webcam.next().await {
                    Some(data) => {
                        // info!("sending sample");
                        let sample = Sample {
                            data: data,
                            duration: Duration::from_secs(1),
                            ..Default::default()
                        };

                        match video_track.write_sample(&sample).await {
                            Ok(_) => {}
                            Err(_) => return,
                        }
                        // info!("sent sample");
                    }
                    None => {
                        info!("webcam stream ended");
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

pub(crate) trait RoomHandler {
    async fn create_and_connect_to_room(&mut self) -> Result<String, Error>;
    async fn join_and_connect_to_room(&mut self, room_id: &str) -> Result<(), Error>;
    async fn connect_to_room(&self, token: &str) -> Result<(WriteStream, MessageReceiver), Error>;
    async fn send_chat(&mut self, content: &str) -> Result<(), Error>;
}

pub(crate) trait SignalHandler {
    async fn send_offer(&mut self) -> Result<(), Error>;
    async fn handle_offer(&mut self, offer: &RTCSessionDescription) -> Result<(), Error>;
    async fn handle_answer(&mut self, answer: &RTCSessionDescription) -> Result<(), Error>;
    async fn handle_candidate(&mut self, candidate: &RTCIceCandidateInit) -> Result<(), Error>;
}
