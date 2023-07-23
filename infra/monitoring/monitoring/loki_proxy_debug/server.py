import logging
from http.server import SimpleHTTPRequestHandler, HTTPServer

logging.basicConfig(level=logging.INFO)

class RequestHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        logging.info(f"GET request,\nPath: {str(self.path)}\nHeaders: {str(self.headers)}")
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"GET request received\n")

    def do_POST(self):
        content_length = int(self.headers['Content-Length']) # Get the size of data
        post_data = self.rfile.read(content_length) # Get the data
        logging.info(f"POST request,\nPath: {str(self.path)}\nHeaders: {str(self.headers)}\n\nBody:\n{post_data.decode('utf-8')}")
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"POST request received\n")

def run(server_class=HTTPServer, handler_class=RequestHandler):
    server_address = ('', 4101)
    httpd = server_class(server_address, handler_class)
    httpd.serve_forever()

if __name__ == "__main__":
    run()