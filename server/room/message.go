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
	MessageTypeChat  RoomMessageType = "chat"
	MessageTypeJoin  RoomMessageType = "join"
	MessageTypeLeave RoomMessageType = "leave"
)

func newJoinMessage(username string) RoomMessage {
	return RoomMessage{
		Type:     MessageTypeJoin,
		Username: username,
		Content:  nil,
	}
}

func newLeaveMessage(username string) RoomMessage {
	return RoomMessage{
		Type:     MessageTypeLeave,
		Username: username,
		Content:  nil,
	}
}

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
