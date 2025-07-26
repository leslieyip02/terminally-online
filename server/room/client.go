package room

import (
	"log"

	"github.com/gorilla/websocket"
)

type Client struct {
	username string
	conn     *websocket.Conn
	room     *Room
	send     chan []byte
}

func (c *Client) readPump() {
	defer func() {
		c.room.unregister <- c
		c.conn.Close()
	}()

	for {
		_, data, err := c.conn.ReadMessage()
		if err != nil {
			break
		}

		message, err := deserializeMessage(data)
		if err != nil {
			log.Printf("[client %s] failed to deserialize %s", c.username, err)
			continue
		}
		
		switch message.Type {
			case MessageTypeChat, MessageTypeJoin, MessageTypeLeave:
				c.room.broadcastToAll(data)

			case MessageTypeOffer, MessageTypeAnswer, MessageTypeCandidate:
				c.room.broadcastExcluding(data, c)

			default:
				continue
		}
	}
}

func (c *Client) writePump() {
	defer c.conn.Close()

	for msg := range c.send {
		if err := c.conn.WriteMessage(websocket.TextMessage, msg); err != nil {
			break
		}
	}
}
