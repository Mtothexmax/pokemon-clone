"""Serves the web export like GitHub Pages would, for testing it locally or on a phone in the same Wi-Fi: the local
export (export.ps1 -Local -> out/web-local/) when there is one, else docs/ (or docs/ anyway with --docs).

- threaded, with a long listen queue: the game fetches hundreds of files at once (python -m http.server queues only 5
  connections and refuses the rest, which shows up as random "file read error" for props and signs)
- sends the COOP/COEP headers itself, so the page is cross-origin isolated even before coi-sw.js takes over

Run from the project root:
  python tools/serve_docs.py [port]            http://localhost:8780/ (this PC only: localhost counts as secure)
  python tools/serve_docs.py --lan [port]      https://<this PC's Wi-Fi address>:8443/ for a phone in the same Wi-Fi

A phone needs HTTPS that it TRUSTS: browsers only allow WebGPU in a secure context, and they refuse to start the
game's workers from a page whose certificate warning was merely clicked through. So --lan makes a small certificate
authority of its own (tools/.lan-cert/, openssl, never committed) and signs this PC's certificate with it. Install the
authority's certificate on the phone once (both servers offer it at /brickhollow-test-ca.crt):
  Android: Settings > Security > Encryption & credentials > Install a certificate > CA certificate
  iPhone:  open the link in Safari, Settings > Profile Downloaded > Install, then
           Settings > General > About > Certificate Trust Settings > switch it on
The authority is name-constrained: it can only vouch for private addresses (10/8, 172.16/12, 192.168/16, 127/8,
localhost), never for real websites, and its key stays on this PC. Remove it from the phone when you stop testing.
Windows may ask to let Python through the firewall for private networks the first time.
"""
import functools
import os
import socket
import ssl
import subprocess
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

PROJECT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# what it serves: the local export (export.ps1 -Local, git-ignored) when there is one, else the GitHub Pages docs/;
# --docs serves docs/ anyway
LOCAL = os.path.join(PROJECT, 'out', 'web-local')
ROOT = LOCAL if os.path.exists(os.path.join(LOCAL, 'index.html')) and '--docs' not in sys.argv else os.path.join(PROJECT, 'docs')
CERT_DIR = os.path.join(PROJECT, 'tools', '.lan-cert')
CA_CRT = os.path.join(CERT_DIR, 'brickhollow-test-ca.crt')

CA_CONFIG = """[req]
distinguished_name = dn
prompt = no
x509_extensions = v3
[dn]
CN = Brickhollow local test CA
[v3]
basicConstraints = critical,CA:true,pathlen:0
keyUsage = critical,keyCertSign,cRLSign
subjectKeyIdentifier = hash
nameConstraints = critical,permitted;IP:10.0.0.0/255.0.0.0,permitted;IP:172.16.0.0/255.240.0.0,permitted;IP:192.168.0.0/255.255.0.0,permitted;IP:127.0.0.0/255.0.0.0,permitted;DNS:localhost
"""


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

    def do_GET(self):
        # the test authority's certificate, for installing it on the phone
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
    """this PC's IPv4 addresses in the local network (no traffic leaves the PC)"""
    ips = set()
    try:
        for ip in socket.gethostbyname_ex(socket.gethostname())[2]:
            if not ip.startswith('127.') and not ip.startswith('169.254.'):
                ips.add(ip)
    except OSError:
        pass
    return sorted(ips)


def openssl(*args):
    subprocess.run(['openssl', *args], check=True, capture_output=True)


def ensure_ca():
    """the local authority: name-constrained to private addresses, valid 2 years"""
    key = os.path.join(CERT_DIR, 'ca.key')
    if os.path.exists(CA_CRT) and os.path.exists(key):
        return CA_CRT, key
    os.makedirs(CERT_DIR, exist_ok=True)
    cfg = os.path.join(CERT_DIR, 'ca.cnf')
    with open(cfg, 'w') as f:
        f.write(CA_CONFIG)
    openssl('req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '730', '-keyout', key, '-out', CA_CRT, '-config', cfg)
    return CA_CRT, key


def ensure_cert(ips):
    """this PC's certificate for its addresses, signed by the local authority (made again when they change)"""
    ca_crt, ca_key = ensure_ca()
    cert, key, stamp = (os.path.join(CERT_DIR, n) for n in ('cert.pem', 'key.pem', 'addresses.txt'))
    want = ','.join(ips)
    if os.path.exists(cert) and os.path.exists(key) and os.path.exists(stamp) and open(stamp).read() == want:
        return cert, key
    csr, ext = os.path.join(CERT_DIR, 'cert.csr'), os.path.join(CERT_DIR, 'cert.ext')
    san = ','.join(['DNS:localhost', 'IP:127.0.0.1'] + [f'IP:{ip}' for ip in ips])
    with open(ext, 'w') as f:
        f.write('basicConstraints = CA:false\n'
                'keyUsage = critical,digitalSignature,keyEncipherment\n'
                'extendedKeyUsage = serverAuth\n'
                f'subjectAltName = {san}\n')
    openssl('req', '-newkey', 'rsa:2048', '-nodes', '-keyout', key, '-out', csr, '-subj', '/CN=Brickhollow local test')
    # 397 days at most: phones refuse longer-lived server certificates
    openssl('x509', '-req', '-in', csr, '-CA', ca_crt, '-CAkey', ca_key, '-CAcreateserial', '-days', '397', '-out', cert,
            '-extfile', ext)
    with open(stamp, 'w') as f:
        f.write(want)
    return cert, key


if __name__ == '__main__':
    args = sys.argv[1:]
    lan = '--lan' in args
    args = [a for a in args if a not in ('--lan', '--docs')]
    port = int(args[0]) if args else (8443 if lan else 8780)
    httpd = Server(('', port), functools.partial(Handler, directory=ROOT))
    if lan:
        ips = lan_addresses()
        cert, key = ensure_cert(ips)
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.load_cert_chain(cert, key)
        httpd.socket = ctx.wrap_socket(httpd.socket, server_side=True)
        print(f'serving {ROOT} over HTTPS. On the phone (same Wi-Fi), once: install the test certificate from', flush=True)
        for ip in ips:
            print(f'  http://{ip}:8780/brickhollow-test-ca.crt   (the plain server must run too)', flush=True)
        print('then open:', flush=True)
        for ip in ips:
            print(f'  https://{ip}:{port}/', flush=True)
    else:
        print(f'serving {ROOT} on http://localhost:{port}/', flush=True)
    httpd.serve_forever()
