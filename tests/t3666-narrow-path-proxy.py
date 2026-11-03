#!/usr/bin/env python3
"""Throttling TCP proxy: client->upstream direction is read slowly with a tiny
receive buffer so the client's socket send buffer fills (mimics a narrow SSH
forward window). upstream->client is passed straight through."""
import socket, sys, threading, time
lport, uhost, uport = int(sys.argv[1]), sys.argv[2], int(sys.argv[3])
chunk = int(sys.argv[4]) if len(sys.argv) > 4 else 4096
delay = float(sys.argv[5]) if len(sys.argv) > 5 else 0.002
def pipe(a, b, slow):
    try:
        while True:
            d = a.recv(chunk if slow else 65536)
            if not d: break
            b.sendall(d)
            if slow: time.sleep(delay)
    except OSError: pass
    finally:
        try: b.shutdown(socket.SHUT_WR)
        except OSError: pass
s = socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4096)
s.setsockopt(socket.IPPROTO_TCP, socket.TCP_MAXSEG, int(sys.argv[6]) if len(sys.argv) > 6 else 1400)
s.bind(("127.0.0.1", lport)); s.listen(16)
while True:
    c, _ = s.accept()
    c.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4096)
    u = socket.create_connection((uhost, uport))
    threading.Thread(target=pipe, args=(c, u, True), daemon=True).start()
    threading.Thread(target=pipe, args=(u, c, False), daemon=True).start()
