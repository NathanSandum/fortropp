import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Echo(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def answer(self):
        length = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(length).decode("latin1") if length else ""
        if self.path.endswith("/redirect"):
            self.send_response(302)
            self.send_header("Location", "https://elsewhere.example/landing")
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        payload = json.dumps(
            {
                "method": self.command,
                "path": self.path,
                "headers": [[k, v] for k, v in self.headers.items()],
                "body": body,
            }
        ).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Set-Cookie", "upstream=1; Path=/")
        self.send_header("X-Upstream", "echo")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(payload)

    do_GET = do_POST = do_PUT = do_PATCH = do_DELETE = do_OPTIONS = do_HEAD = answer


ThreadingHTTPServer(("0.0.0.0", 8080), Echo).serve_forever()
