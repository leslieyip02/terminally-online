package signaling

import (
	"log"

	"github.com/pion/webrtc/v4"
)

type ConnectionManager struct {
	peerConnection *webrtc.PeerConnection
}

func NewConnectionManager() (*ConnectionManager, error) {
	config := webrtc.Configuration{
		ICEServers: []webrtc.ICEServer{
			{
				URLs: []string{"stun:stun.l.google.com:19302"},
			},
		},
	}
	peerConnection, err := webrtc.NewPeerConnection(config)
	if err != nil {
		return nil, err
	}

	peerConnection.OnTrack(func(tr *webrtc.TrackRemote, r *webrtc.RTPReceiver) {
		log.Println("Received track", tr.Kind().String())
	})

	connectionManager := &ConnectionManager{
		peerConnection: peerConnection,
	}
	return connectionManager, nil
}

func (m *ConnectionManager) Init(client CandidateSink) {
	m.peerConnection.OnICECandidate(func(c *webrtc.ICECandidate) {
		if c == nil {
			return
		}

		payload := c.ToJSON().Candidate
		candidateMessage := SignalMessage{
			Type:    SignalMessageTypeCandidate,
			Payload: &payload,
		}
		client.OnCandidateMessage(&candidateMessage)
	})
}

func (m *ConnectionManager) HandleOffer(sdp string) (*SignalMessage, error) {
	offer := webrtc.SessionDescription{
		Type: webrtc.SDPTypeOffer,
		SDP:  sdp,
	}
	if err := m.peerConnection.SetRemoteDescription(offer); err != nil {
		return nil, err
	}

	answer, err := m.peerConnection.CreateAnswer(nil)
	if err != nil {
		return nil, err
	}

	gatherComplete := webrtc.GatheringCompletePromise(m.peerConnection)
	if err := m.peerConnection.SetLocalDescription(answer); err != nil {
		log.Println("error setting local description:", err)
		return nil, err
	}
	<-gatherComplete

	payload := m.peerConnection.CurrentLocalDescription().SDP
	message := SignalMessage{
		Type:    SignalMessageTypeAnswer,
		Payload: &payload,
	}
	return &message, nil
}

func (m *ConnectionManager) HandleCandidate(candidate string) error {
	return m.peerConnection.AddICECandidate(webrtc.ICECandidateInit{
		Candidate: candidate,
	})
}
