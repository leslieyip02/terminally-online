package room

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
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

		case client := <-r.unregister:
			if _, ok := r.clients[client]; ok {
				delete(r.clients, client)
				close(client.send)
			}

		case data := <-r.broadcast:
			r.log(fmt.Sprintf("received %s", string(data)))
			message, err := parseMessage(data)
			if err != nil {
				r.log(fmt.Sprintf("parsing error %s", err))
			}

			if message.Type == MessageTypeChat {
				for client := range r.clients {
					select {
					case client.send <- data:
					default:
						close(client.send)
						delete(r.clients, client)
					}
				}
			}
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

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

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
	roomId := r.URL.Query().Get("room")
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

	client := &Client{conn: conn, send: make(chan []byte, 256), room: room}
	room.register <- client

	go client.readPump()
	go client.writePump()
}

func (m *RoomManager) createRoom() *Room {
	m.mu.Lock()
	defer m.mu.Unlock()

	room := &Room{
		roomId:     uuid.New().String(),
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
