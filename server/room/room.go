package room

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"github.com/sqids/sqids-go"
)

type Room struct {
	roomId     string
	clients    map[*Client]bool
	broadcast  chan []byte
	register   chan *Client
	unregister chan *Client
}

func (r *Room) run() {
	for {
		select {
		case client := <-r.register:
			r.clients[client] = true
			r.broadcastMessage(newJoinMessage(client.username))

		case client := <-r.unregister:
			if _, ok := r.clients[client]; ok {
				delete(r.clients, client)
				close(client.send)
			}
			r.broadcastMessage(newLeaveMessage(client.username))

		case data := <-r.broadcast:
			r.log(fmt.Sprintf("received %s", string(data)))
			message, err := deserializeMessage(data)
			if err != nil {
				r.log(fmt.Sprintf("parsing error %s", err))
			}

			switch message.Type {
			case MessageTypeChat, MessageTypeJoin, MessageTypeLeave:
				r.broadcastMessage(message)
			default:
				continue
			}
		}
	}
}

func (r *Room) broadcastMessage(message RoomMessage) {
	data, err := serializeMessage(&message)
	if err != nil {
		return
	}

	for client := range r.clients {
		select {
		case client.send <- data:
		default:
			close(client.send)
			delete(r.clients, client)
			r.broadcastMessage(newLeaveMessage(client.username))
		}
	}
}

func (r *Room) log(message string) {
	log.Printf("[room %s] %s", r.roomId, message)
}

type RoomManager struct {
	rooms          map[string]*Room
	mu             sync.Mutex
	sessionManager *SessionManager
}

var (
	upgrader = websocket.Upgrader{
		CheckOrigin: func(r *http.Request) bool { return true },
	}
	encoder, _ = sqids.New(sqids.Options{MinLength: 6})
)

func NewRoomManager(sessionManager *SessionManager) *RoomManager {
	return &RoomManager{
		rooms:          make(map[string]*Room),
		sessionManager: sessionManager,
	}
}

func (m *RoomManager) HandleCreateRoom(w http.ResponseWriter, r *http.Request) {
	// TODO: check if user is already in a room
	room := m.createRoom()

	token, err := m.sessionManager.createToken(room.roomId)
	if err != nil {
		http.Error(w, "unable to issue token", http.StatusInternalServerError)
		return
	}

	body := map[string]string{
		"room":  room.roomId,
		"token": token,
	}

	if err := json.NewEncoder(w).Encode(body); err != nil {
		http.Error(w, "unable to create room", http.StatusInternalServerError)
	}
}

func (m *RoomManager) HandleJoinRoom(w http.ResponseWriter, r *http.Request) {
	roomId := r.PathValue("room")
	_, ok := m.rooms[roomId]
	if !ok {
		http.Error(w, "invalid room", http.StatusNotFound)
		return
	}

	token, err := m.sessionManager.createToken(roomId)
	if err != nil {
		http.Error(w, "unable to issue token", http.StatusInternalServerError)
		return
	}

	body := map[string]string{
		"room":  roomId,
		"token": token,
	}

	if err := json.NewEncoder(w).Encode(body); err != nil {
		http.Error(w, "unable to join room", http.StatusInternalServerError)
	}
}

func (m *RoomManager) HandleWebSocket(w http.ResponseWriter, r *http.Request) {
	token := r.URL.Query().Get("token")
	roomId, err := m.sessionManager.validateToken(token)
	if err != nil {
		http.Error(w, "invalid token", http.StatusUnauthorized)
		return
	}

	var username = r.URL.Query().Get("username")
	if username == "" {
		username = "???"
	}

	room, ok := m.rooms[roomId]
	if !ok {
		http.Error(w, "invalid room", http.StatusNotFound)
		return
	}

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		http.Error(w, "unable to create connection", http.StatusInternalServerError)
		return
	}

	client := &Client{
		username: username,
		conn:     conn,
		send:     make(chan []byte, 256),
		room:     room,
	}
	room.register <- client

	go client.readPump()
	go client.writePump()
}

func (m *RoomManager) createRoom() *Room {
	m.mu.Lock()
	defer m.mu.Unlock()

	// TODO: feels hacky? should be ok for prototyping
	var roomId, err = randomRoomId()
	var _, ok = m.rooms[roomId]
	for ok || err != nil {
		log.Println(err)
		roomId, err = randomRoomId()
		_, ok = m.rooms[roomId]
	}

	room := &Room{
		roomId:     roomId,
		clients:    make(map[*Client]bool),
		broadcast:  make(chan []byte),
		register:   make(chan *Client),
		unregister: make(chan *Client),
	}
	m.rooms[room.roomId] = room
	room.log("created")

	go room.run()
	return room
}

func randomRoomId() (string, error) {
	timestamp := uint64(time.Now().Unix())
	return encoder.Encode([]uint64{timestamp})
}
