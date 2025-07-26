package signaling

import (
	"encoding/json"
)

type SignalMessage struct {
	Type    SignalMessageType `json:"type"`
	Payload any               `json:"payload"`
}

type SignalMessageType string

const (
	SignalMessageTypeOffer     SignalMessageType = "offer"
	SignalMessageTypeAnswer    SignalMessageType = "answer"
	SignalMessageTypeCandidate SignalMessageType = "candidate"
)

func IsSignalMessage(data []byte) bool {
	var probe struct {
		Type string `json:"type"`
	}
	_ = json.Unmarshal(data, &probe)

	switch probe.Type {
	case string(SignalMessageTypeOffer),
		string(SignalMessageTypeAnswer),
		string(SignalMessageTypeCandidate):
		return true
	default:
		return false
	}
}
