package room

import (
	"encoding/json"
	"fmt"
)

type Message struct {
	Type    MessageType `json:"type"`
	User    *string     `json:"user,omitempty"`
	Content string      `json:"content"`
}

type MessageType string

const (
	MessageTypeChat  MessageType = "chat"
	MessageTypeJoin  MessageType = "join"
	MessageTypeLeave MessageType = "leave"
)

func parseMessage(data []byte) (Message, error) {
	var message Message
	if err := json.Unmarshal(data, &message); err != nil {
		return message, err
	}

	switch message.Type {
	case MessageTypeChat:
		return message, nil
	default:
		return message, fmt.Errorf("invalid message type")
	}
}
