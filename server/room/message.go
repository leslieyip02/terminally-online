package room

import (
	"encoding/json"
	"fmt"
)

type RoomMessage struct {
	Type     RoomMessageType `json:"type"`
	Username string          `json:"username"`
	Content  *string         `json:"content,omitempty"`
}

type RoomMessageType string

const (
	MessageTypeChat      RoomMessageType = "chat"
	MessageTypeJoin      RoomMessageType = "join"
	MessageTypeLeave     RoomMessageType = "leave"
	MessageTypeOffer     RoomMessageType = "offer"
	MessageTypeAnswer    RoomMessageType = "answer"
	MessageTypeCandidate RoomMessageType = "candidate"
)

func deserializeMessage(data []byte) (RoomMessage, error) {
	var message RoomMessage
	if err := json.Unmarshal(data, &message); err != nil {
		return message, err
	}

	switch message.Type {
	case MessageTypeChat, MessageTypeJoin, MessageTypeLeave:
		return message, nil
	default:
		return message, fmt.Errorf("invalid message type")
	}
}

func serializeMessage(message *RoomMessage) ([]byte, error) {
	return json.Marshal(message)
}
