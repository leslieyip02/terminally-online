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
