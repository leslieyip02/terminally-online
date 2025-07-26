package room

import (
	"encoding/json"
	"fmt"
)

type RoomMessage struct {
	Type    RoomMessageType `json:"type"`
	User    string          `json:"user"`
	Content *string         `json:"content,omitempty"`
}

type RoomMessageType string

const (
	MessageTypeChat  RoomMessageType = "chat"
	MessageTypeJoin  RoomMessageType = "join"
	MessageTypeLeave RoomMessageType = "leave"
)

func newChatMessage(user string, content string) RoomMessage {
	return RoomMessage{
		Type:    MessageTypeChat,
		User:    user,
		Content: &content,
	}
}

func newJoinMessage(user string) RoomMessage {
	return RoomMessage{
		Type:    MessageTypeJoin,
		User:    user,
		Content: nil,
	}
}

func newLeaveMessage(user string) RoomMessage {
	return RoomMessage{
		Type:    MessageTypeLeave,
		User:    user,
		Content: nil,
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
