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
		log.Println("could not load .env")
	}

	port := ":" + os.Getenv("PORT")
	jwtSecret := os.Getenv("JWT_SECRET")

	sessionManager := room.NewSessionManager([]byte(jwtSecret))
	roomManager := room.NewRoomManager(&sessionManager)

	mux := http.NewServeMux()
	mux.HandleFunc("POST /create", func(w http.ResponseWriter, r *http.Request) {
		log.Println("[router] POST /create")
		roomManager.HandleCreateRoom(w, r)
	})
	mux.HandleFunc("POST /join/{room}", func(w http.ResponseWriter, r *http.Request) {
		log.Println("[router] POST /join/{room}")
		roomManager.HandleJoinRoom(w, r)
	})
	mux.HandleFunc("GET /ws", func(w http.ResponseWriter, r *http.Request) {
		log.Println("[router] GET /ws")
		roomManager.HandleWebSocket(w, r)
	})

	log.Printf("Server started on %s", port)
	log.Fatal(http.ListenAndServe(port, mux))
}
