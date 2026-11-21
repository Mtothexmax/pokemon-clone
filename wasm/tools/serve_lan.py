"""Serves a release build of Brickhollow (wasm) over HTTPS for a phone in the same Wi-Fi.

  node C:/dev/wasmOnly/tools/dev.mjs build --game-dir C:/dev/pokemonsnap/wasm --release
  then copy dist/ to out/lan/ (tools/lan.ps1 does both) and run:
  python tools/serve_lan.py [port]        https://<this PC's Wi-Fi address>:8444/

WebGPU needs a secure context, so the phone must load the page over HTTPS with a certificate it trusts. This reuses
the small local certificate authority of the Kaigen port (pokemontest/tools/.lan-cert/, made by its serve_docs.py
--lan, never committed): a phone that already plays the Kaigen build over Wi-Fi trusts this server too. Its
authority certificate is offered at /brickhollow-test-ca.crt for a phone that does not have it yet.
"""

import functools
import os
import socket
import ssl
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(os.path.dirname(HERE), 'out', 'lan')
CERT_DIR = os.path.join(os.environ.get('LOCALAPPDATA', ''), 'KaigenProjects', 'pokemontest', 'tools', '.lan-cert')
CA_CRT = os.path.join(CERT_DIR, 'brickhollow-test-ca.crt')
# what the phone's console sends (tools/phone_log.js)
PHONE_LOG = os.path.join(os.path.dirname(HERE), 'logs', 'phone.log')  # (logs/: the dev watcher ignores it)


class Handler(SimpleHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, '.js': 'text/javascript', '.wasm': 'application/wasm',
                      '.mjs': 'text/javascript', '.glb': 'model/gltf-binary', '.ktx2': 'image/ktx2'}

    def end_headers(self):
        self.send_header('Cache-Control', 'no-cache')
        super().end_headers()

    def do_POST(self):
        if self.path == '/__log':
            n = int(self.headers.get('Content-Length') or 0)
            line = self.rfile.read(n).decode('utf-8', 'replace')
            with open(PHONE_LOG, 'a', encoding='utf-8') as f:
                f.write(f'{self.client_address[0]} {line}' + '\n')
            self.send_response(204)
            self.send_header('Content-Length', '0')
            self.end_headers()
            return
        self.send_error(404)

    def do_GET(self):
        if self.path.split('?')[0] == '/brickhollow-test-ca.crt' and os.path.exists(CA_CRT):
            with open(CA_CRT, 'rb') as f:
                data = f.read()
            self.send_response(200)
            self.send_header('Content-Type', 'application/x-x509-ca-cert')
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)
            return
        super().do_GET()

    def log_message(self, fmt, *args):
        if not str(args[1] if len(args) > 1 else '').startswith(('2', '3')):
            super().log_message(fmt, *args)  # only problems


class Server(ThreadingHTTPServer):
    request_queue_size = 256
    daemon_threads = True

    def handle_error(self, request, client_address):
        pass  # a phone dropping a connection is not worth a traceback


def lan_addresses():
    ips = set()
    try:
        for ip in socket.gethostbyname_ex(socket.gethostname())[2]:
            if not ip.startswith('127.') and not ip.startswith('169.254.'):
                ips.add(ip)
    except OSError:
        pass
    return sorted(ips)


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8444
    cert, key = os.path.join(CERT_DIR, 'cert.pem'), os.path.join(CERT_DIR, 'key.pem')
    if not (os.path.exists(cert) and os.path.exists(key)):
        sys.exit(f'no certificate in {CERT_DIR}: run the Kaigen port once with "python tools/serve_docs.py --lan"')
    if not os.path.exists(os.path.join(ROOT, 'index.html')):
        sys.exit(f'nothing to serve in {ROOT}: run tools/lan.ps1 first')
    httpd = Server(('', port), functools.partial(Handler, directory=ROOT))
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(cert, key)
    httpd.socket = ctx.wrap_socket(httpd.socket, server_side=True)
    print(f'serving {ROOT} over HTTPS; on the phone (same Wi-Fi) open:', flush=True)
    for ip in lan_addresses():
        print(f'  https://{ip}:{port}/', flush=True)
    httpd.serve_forever()


if __name__ == '__main__':
    main()
