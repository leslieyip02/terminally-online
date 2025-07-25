# Client

## Quick Start

1. Allow webcam access from the terminal

2. Start the client

```
cargo run
```

### WSL Compatibility

Attach webcam to WSL client:

```
# Command Prompt
usbipd attach --wsl --busid <busid>

# WSL
modprobe uvcvideo
chgrp video /dev/video0
chmod 660 /dev/video0
```
