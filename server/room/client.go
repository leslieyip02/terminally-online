package room

import (
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
			r.room.broadcastToAllExcept(data, r)
		} else {
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
