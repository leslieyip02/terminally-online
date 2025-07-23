# Terminally Online

> Zoom but worse

## WSL Compatiblity

```
# Command Prompt
usbipd attach --wsl --busid <busid>

# WSL
modprobe uvcvideo
chgrp video /dev/video0
chmod 660 /dev/video0
```
