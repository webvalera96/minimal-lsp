#!/usr/bin/env python3
"""
Mock LSP client – sends the same nine-packet LSP test as rust-analyzer's
manual_test.sh but over a named pipe (FIFO).

Usage (two terminals):

  mkfifo /tmp/lsp_pipe       # one-time setup
  # tab 1 – run the server
  cat /tmp/lsp_pipe | python -m minimal_lsp   # or whatever runs your LSP

  # tab 2 – fire the packets (this script)
  python main.py              # blocks until server exits

If you don't use a second tab, run in background:

  python main.py &
  cat /tmp/lsp_pipe | python -m minimal_lsp
"""

import io
import json
import os
import sys

DEFAULT_PIPE = "/tmp/lsp_pipe"


def send_body(fd: io.BufferedWriter, body: dict) -> None:
    """Send a JSON-RPC packet with a Content-Length header over fd."""
    payload = json.dumps(body, separators=(",", ":"))
    header = f"Content-Length: {len(payload)}\r\n\r\n"
    fd.write(header.encode("ascii"))
    fd.write(payload.encode("utf-8"))


packets = [
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
]


def main() -> None:
    pipe_path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_PIPE

    os.mkfifo(pipe_path, mode=0o600)

    with open(pipe_path, "wb") as fd:
        for pkt in packets:
            send_body(fd, pkt)

    print("Packets sent – watch the other terminal for responses.")


if __name__ == "__main__":
    main()
