package room

type RoomMessage struct {
	Type     RoomMessageType `json:"type"`
	Username *string         `json:"username,omitempty"`
	Content  *string         `json:"content,omitempty"`
}

type RoomMessageType string

const (
	RoomMessageTypeChat  RoomMessageType = "chat"
	RoomMessageTypeJoin  RoomMessageType = "join"
	RoomMessageTypeLeave RoomMessageType = "leave"
)
