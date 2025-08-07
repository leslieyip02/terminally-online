# Server

## Quick Start

1. Create an `.env` file:

```
PORT=<port>
JWT_SECRET=<secret>
```

2. Start the server:

```
go run main.go
```

## Implementation

### Rooms

Messages between clients are transmitted using [WebSockets](https://en.wikipedia.org/wiki/WebSocket), more specifically [Gorilla WebSocket](https://github.com/gorilla/websocket).
A client can create a room by typing the `/create` command in the terminal, which sends a `POST` request to the `/create` endpoint.
This returns a room ID and a [JWT](https://en.wikipedia.org/wiki/JSON_Web_Token).
Other clients can then join the same room by typing `/join ID`, which sends a `POST` request to `/join/{ID}`.
This also issues a JWT for authentication.

After receiving the JWT, the client will attempt to establish a WebSocket connection to the room.
Once connected, they can then send chat messages which are broadcast to all other users in the same room.

### Peer-to-Peer Connections

[WebRTC](https://en.wikipedia.org/wiki/WebRTC) is used to establish peer-to-peer media streams between clients.
The server acts as a signaling broker between clients, facilitating the exchange of offer, answer, and ICE candidate messages necessary to set up the connection.

The connection flow is as follows:

1. Client A sends an offer
2. Client B receives the offer and replies with an answer
3. Both clients exchange [ICE candidates](https://developer.mozilla.org/en-US/docs/Web/API/RTCIceCandidate)
4. Clients negotiate and agree on a connection method
5. Once the connection is established, media streams are exchanged directly between clients

## Hosting on [alwaysdata](https://www.alwaysdata.com/en/)

1. Compile executable:

```
GOOS=linux GOARCH=amd64 go build -o server main.go
```

2. Upload executable:

```
scp -P 22 server <user>@<host>:<destination>
```

3. Configure environment variables in the site's dashboard
