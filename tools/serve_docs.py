"""Serves docs/ on http://localhost:8780/ like GitHub Pages would, for testing the web export locally.

- threaded, with a long listen queue: the game fetches hundreds of files at once (python -m http.server queues only 5
  connections and refuses the rest, which shows up as random "file read error" for props and signs)
- sends the COOP/COEP headers itself, so the page is cross-origin isolated even before coi-sw.js takes over

Run from the project root:  python tools/serve_docs.py [port]
"""
import functools
import os
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

ROOT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'docs')


class Handler(SimpleHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, '.js': 'text/javascript', '.wasm': 'application/wasm',
                      '.mjs': 'text/javascript'}

    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Resource-Policy', 'cross-origin')
        self.send_header('Cache-Control', 'no-cache')
        super().end_headers()

    def log_message(self, fmt, *args):
        if not str(args[1] if len(args) > 1 else '').startswith(('2', '3')):
            super().log_message(fmt, *args)  # only problems


class Server(ThreadingHTTPServer):
    request_queue_size = 256
    daemon_threads = True


if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8780
    httpd = Server(('', port), functools.partial(Handler, directory=ROOT))
    print(f'serving {ROOT} on http://localhost:{port}/', flush=True)
    httpd.serve_forever()
