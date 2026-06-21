#!/usr/bin/env python3
"""
Mock LSP client – sends nine JSON-RPC packets to an LSP server over TCP.

Usage:
    python main.py              # connect to 127.0.0.1:9090
    python main.py 192.168.1.10 9999  # custom host/port
"""

import json
import socket
import sys


DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT = 9090


def send_body(sock: socket.socket, body: dict) -> None:
    """Send a JSON-RPC packet with a Content-Length header over sock."""
    payload = json.dumps(body, separators=(",", ":"))
    header = f"Content-Length: {len(payload)}\r\n\r\n"
    sock.sendall(header.encode("ascii"))
    sock.sendall(payload.encode("utf-8"))


def main() -> None:
    host = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_HOST
    port = int(sys.argv[2]) if len(sys.argv) > 2 else DEFAULT_PORT

    with socket.create_connection((host, port)) as sock:
        for pkt in [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {"capabilities": {}},
            },
            {
                "jsonrpc": "2.0",
                "method": "initialized",
                "params": {},
            },
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///tmp/foo.rs",
                        "languageId": "rust",
                        "version": 1,
                        "text": 'fn  main( ){println!("hi") }',
                    }
                },
            },
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "textDocument/completion",
                "params": {
                    "textDocument": {"uri": "file:///tmp/foo.rs"},
                    "position": {"line": 0, "character": 0},
                },
            },
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "textDocument/hover",
                "params": {
                    "textDocument": {"uri": "file:///tmp/foo.rs"},
                    "position": {"line": 0, "character": 0},
                },
            },
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "textDocument/definition",
                "params": {
                    "textDocument": {"uri": "file:///tmp/foo.rs"},
                    "position": {"line": 0, "character": 0},
                },
            },
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "textDocument/formatting",
                "params": {
                    "textDocument": {"uri": "file:///tmp/foo.rs"},
                    "options": {"tabSize": 4, "insertSpaces": True},
                },
            },
            {
                "jsonrpc": "2.0",
                "id": 6,
                "method": "shutdown",
                "params": None,
            },
            {
                "jsonrpc": "2.0",
                "method": "exit",
                "params": None,
            },
        ]:
            send_body(sock, pkt)

    print(f"Packets sent to {host}:{port}")


if __name__ == "__main__":
    main()
