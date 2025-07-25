package main

import (
	"log"
	"net/http"
	"os"

	"server/room"

	"github.com/joho/godotenv"
)

func main() {
	err := godotenv.Load()
	if err != nil {
		log.Fatal("unable to load .env")
		return
	}
	jwtSecret := os.Getenv("JWT_SECRET")

	sessionManager := room.NewSessionManager([]byte(jwtSecret))
	roomManager := room.NewRoomManager(&sessionManager)

	http.HandleFunc("/create", func(w http.ResponseWriter, r *http.Request) {
		roomManager.HandleCreateRoom(w, r)
	})

	http.HandleFunc("/join", func(w http.ResponseWriter, r *http.Request) {
		roomManager.HandleJoinRoom(w, r)
	})

	http.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		roomManager.HandleWebSocket(w, r)
	})

	log.Println("Server started on :8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
