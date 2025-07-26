package room

import (
	"encoding/json"
	"log"

	"server/signaling"
)

type Room struct {
	roomId            string
	clients           map[*RoomClient]bool
	register          chan *RoomClient
	unregister        chan *RoomClient
	connectionManager signaling.ConnectionManager
}

func NewRoom(roomId string) (*Room, error) {
	connectionManager, err := signaling.NewConnectionManager()
	if err != nil {
		return nil, err
	}

	room := Room{
		roomId:            roomId,
		clients:           make(map[*RoomClient]bool),
		register:          make(chan *RoomClient),
		unregister:        make(chan *RoomClient),
		connectionManager: *connectionManager,
	}
	return &room, nil
}

func (r *Room) run() {
	log.Printf("[room %s] running", r.roomId)

	for {
		select {
		case client := <-r.register:
			r.clients[client] = true
			log.Printf("[room %s] %s joined", r.roomId, client.username)

			joinMessage := RoomMessage{
				Type:     RoomMessageTypeJoin,
				Username: &client.username,
				Content:  nil,
			}
			serialized, err := json.Marshal(&joinMessage)
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
				Type:     RoomMessageTypeLeave,
				Username: &client.username,
				Content:  nil,
			}
			serialized, err := json.Marshal(&leaveMessage)
			if err != nil {
				continue
			}
			r.broadcastToAll(serialized)
		}
	}
}

func (r *Room) broadcastToAll(data []byte) {
	r.broadcastToAllExcept(data, nil)
}

func (r *Room) broadcastToAllExcept(data []byte, exclude *RoomClient) {
	for client := range r.clients {
		if client == exclude {
			continue
		}
		client.send <- data
	}
}

func (r *Room) OnCandidateMessage(message *signaling.SignalMessage) {
	log.Printf("[room %s] received candidate with payload: %s", r.roomId, message.Payload)

	// TODO: implement in the future when dealing with media on the server side
}
