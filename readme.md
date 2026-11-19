# tdsr server
This is a Windows server that takes speech from [tdsr](https://github.com/tspivey/tdsr) and speaks it through your screen reader, such as NVDA or JAWS. If no screen reader is running, it uses a Windows text-to-speech voice instead. Speech goes through [prism](https://github.com/ethindp/prism).

## Download
Download the exe for your PC from the [latest release](https://github.com/trypsynth/tdsr-server/releases/latest). It is a single file with nothing to install.

- [tdsr-server-x64.exe](https://github.com/trypsynth/tdsr-server/releases/latest/download/tdsr-server-x64.exe), for most PCs.
- [tdsr-server-arm64.exe](https://github.com/trypsynth/tdsr-server/releases/latest/download/tdsr-server-arm64.exe), for ARM PCs.

## Usage
Run the exe. It listens for connections on port 64111, or on the port you give it as a command-line argument.

On the machine running tdsr, create a shell script, something like:
```shell
#!/bin/bash
exec socat - TCP4:SERVER_IP:64111,nodelay
```
Replace `SERVER_IP` with the IP address of the server, and 64111 with your custom port, if you specified one.

The server adds an icon to the system tray. To close the server, right-click the icon and choose Quit.

## Building
You need Rust 1.90 or later, CMake 3.24 or later, and Visual Studio's C++ build tools with ATL, which build prism. Then run `cargo build --release`, and the exe is at `target\release\tdsr-server.exe`.

## Security
This program binds to all interfaces on port 64111. The only things it lets you do are to have it speak a string and stop speaking.
