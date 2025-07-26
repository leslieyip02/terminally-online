package room

import (
	"encoding/json"
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"github.com/sqids/sqids-go"
)

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
	room, err := m.createRoom()
	if err != nil {
		http.Error(w, "unable to create room", http.StatusInternalServerError)
		return
	}
	room.connectionManager.Init(room)

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

	client := &RoomClient{
		username: username,
		conn:     conn,
		send:     make(chan []byte, 256),
		room:     room,
	}
	room.register <- client

	go client.readPump()
	go client.writePump()
}

func (m *RoomManager) createRoom() (*Room, error) {
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

	room, err := NewRoom(roomId)
	if err != nil {
		return nil, err
	}
	m.rooms[room.roomId] = room

	go room.run()
	return room, nil
}

func randomRoomId() (string, error) {
	timestamp := uint64(time.Now().Unix())
	return encoder.Encode([]uint64{timestamp})
}
