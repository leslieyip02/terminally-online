# Client

## Quick Start

1. Create an `.env` file:

```
HOST=<host>
```

2. Start the client:

```
cargo run
```

## Compatibility

Tested on WSL and MacOS.

### Additional Setup for WSL

Attach webcam to WSL client:

```
# Command Prompt
usbipd attach --wsl --busid <busid>

# WSL
modprobe uvcvideo
chgrp video /dev/video0
chmod 660 /dev/video0
```
