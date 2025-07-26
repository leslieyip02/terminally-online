package room

import (
	"log"
)

type Room struct {
	roomId     string
	clients    map[*Client]bool
	register   chan *Client
	unregister chan *Client
}

func (r *Room) run() {
	log.Printf("[room %s] running", r.roomId)

	for {
		select {
		case client := <-r.register:
			r.clients[client] = true
			log.Printf("[room %s] %s joined", r.roomId, client.username)

			joinMessage := RoomMessage{
				Type:     MessageTypeJoin,
				Username: client.username,
				Content:  nil,
			}
			serialized, err := serializeMessage(&joinMessage)
			if err != nil {
				continue
			}
			r.broadcastToAll(serialized)

		case client := <-r.unregister:
			if _, ok := r.clients[client]; ok {
				delete(r.clients, client)
				close(client.send)
			}
			log.Printf("[room %s] %s left", r.roomId, client.username)

			leaveMessage := RoomMessage{
				Type:     MessageTypeLeave,
				Username: client.username,
				Content:  nil,
			}
			serialized, err := serializeMessage(&leaveMessage)
			if err != nil {
				continue
			}
			r.broadcastToAll(serialized)
		}
	}
}

func (r *Room) broadcastToAll(data []byte) {
	r.broadcastExcluding(data, nil)
}

func (r *Room) broadcastExcluding(data []byte, exclude *Client) {
	for client := range r.clients {
		if client == exclude {
			continue
		}
		client.send <- data
	}
}
