package room

import (
	"encoding/json"
	"log"
	"server/signaling"

	"github.com/gorilla/websocket"
)

type RoomClient struct {
	username string
	conn     *websocket.Conn
	room     *Room
	send     chan []byte
}

func (r *RoomClient) readPump() {
	defer func() {
		r.room.unregister <- r
		r.conn.Close()
	}()

	for {
		_, data, err := r.conn.ReadMessage()
		if err != nil {
			break
		}

		if signaling.IsSignalMessage(data) {
			var message signaling.SignalMessage
			if err := json.Unmarshal(data, &message); err == nil {
				log.Printf("[client %s] sending %s message with payload %s", r.username, message.Type, message.Payload)
			}

			r.room.broadcastToAllExcept(data, r)
		} else {
			var message RoomMessage
			if err := json.Unmarshal(data, &message); err == nil {
				log.Printf("[client %s] sending %s message with payload %s", r.username, message.Type, *message.Content)
			}

			r.room.broadcastToAll(data)
		}
	}
}

func (r *RoomClient) writePump() {
	defer r.conn.Close()

	for data := range r.send {
		if err := r.conn.WriteMessage(websocket.TextMessage, data); err != nil {
			break
		}
	}
}
