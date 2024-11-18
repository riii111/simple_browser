extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use browser_core::error::Error;
use browser_core::http::HttpResponse;
use noli::net::lookup_host;
use noli::net::SocketAddr;
use noli::net::TcpStream;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        // DNSを使用して名前解決
        let ips = match lookup_host(&host) {
            Ok(ips) => ips,
            Err(e) => {
                return Err(Error::Network(format!(
                    "Failed to find IP addresses: {:#?}",
                    e
                )))
            }
        };

        if ips.len() < 1 {
            return Err(Error::Network("Failed to find IP addresses".to_string()));
        }

        let socket_addr: SocketAddr = (ips[0], port).into();

        // コネクションの確立
        let mut stream = match TcpStream::connect(socket_addr) {
            Ok(stream) => stream,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to connect to TCP stream".to_string(),
                ))
            }
        };

        let mut request = String::from("GET /");
        request.push_str(&path);
        request.push_str(" HTTP/1.1\n");

        // ヘッダの追加
        request.push_str("Host: ");
        request.push_str(&host);
        request.push('\n');
        request.push_str("Accept: test/html\n");
        request.push_str("Connection: close\n"); // リクエスト後に毎回接続を切断
        request.push('\n');

        // リクエストの送信
        let _bytes_written = match stream.write(request.as_bytes()) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to send a request to TCP stream".to_string(),
                ))
            }
        };

        // レスポンスの受信
        let mut received = Vec::new();
        loop {
            let mut buf = [0u8; 4096];
            let bytes_read = match stream.read(&mut buf) {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Err(Error::Network(
                        "Failed to receive a request form TCP stream".to_string(),
                    ))
                }
            };
            // 読み込むバイトがなくなるまで
            if bytes_read == 0 {
                break;
            }
            received.extend_from_slice(&buf[..bytes_read]); // 分割されたレスポンスをつなぎ合わせる
        }

        match core::str::from_utf8(&received) {
            Ok(response) => HttpResponse::new(response.to_string()),
            Err(e) => Err(Error::Network(format!("Invalid received response: {}", e))),
        }
    }
}
