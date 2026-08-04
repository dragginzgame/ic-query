use super::*;
use serde::Deserialize;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::{self, JoinHandle},
};

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct TestPayload {
    value: String,
}

struct TestServer {
    url: Url,
    handle: JoinHandle<()>,
}

impl TestServer {
    fn finish(self) {
        self.handle.join().expect("test HTTP server");
    }
}

#[test]
fn bounded_transport_accepts_valid_json_at_the_exact_byte_limit() {
    let body = br#"{"value":"ok"}"#;
    let server = serve_once(content_length_response(body.len(), body));

    let payload: TestPayload =
        fetch_test_json(&server.url, body.len() as u64).expect("response at exact limit");
    server.finish();

    assert_eq!(
        payload,
        TestPayload {
            value: "ok".to_string()
        }
    );
}

#[test]
fn bounded_transport_rejects_oversized_declared_length_before_reading_body() {
    let server = serve_once(content_length_response(65, &[]));

    let error =
        fetch_test_json::<TestPayload>(&server.url, 64).expect_err("oversized declared response");
    server.finish();

    assert!(matches!(
        error,
        IcHostError::HttpResponseTooLarge {
            max_bytes: 64,
            observed_bytes: 65,
            ..
        }
    ));
}

#[test]
fn bounded_transport_rejects_oversized_chunked_body_without_length_header() {
    let filler = vec![b'x'; 64];
    let server = serve_once(chunked_response(&[br#"{"value":""#, &filler, br#""}"#]));

    let error =
        fetch_test_json::<TestPayload>(&server.url, 32).expect_err("oversized chunked response");
    server.finish();

    assert!(matches!(
        error,
        IcHostError::HttpResponseTooLarge {
            max_bytes: 32,
            observed_bytes,
            ..
        } if observed_bytes > 32
    ));
}

#[test]
fn bounded_transport_distinguishes_body_read_and_json_decode_failures() {
    let truncated = serve_once(content_length_response(100, b"{"));
    let read_error =
        fetch_test_json::<TestPayload>(&truncated.url, 200).expect_err("truncated response body");
    truncated.finish();
    assert!(matches!(read_error, IcHostError::HttpResponseBody { .. }));

    let malformed = serve_once(content_length_response(8, b"not-json"));
    let decode_error =
        fetch_test_json::<TestPayload>(&malformed.url, 8).expect_err("malformed response JSON");
    malformed.finish();
    assert!(matches!(decode_error, IcHostError::JsonDecode { .. }));
}

#[test]
fn dashboard_transport_rejects_redirects_without_following_the_location() {
    let server = serve_once(redirect_response("https://example.com/other-authority"));

    let error =
        fetch_test_json::<TestPayload>(&server.url, 100).expect_err("redirect must be rejected");
    server.finish();

    assert!(matches!(error, IcHostError::HttpStatus { status: 302, .. }));
}

fn fetch_test_json<T>(url: &Url, max_response_bytes: u64) -> Result<T, IcHostError>
where
    T: DeserializeOwned + Send,
{
    block_on_current_thread(fetch_json_with_limit(
        http_client()?,
        url.clone(),
        max_response_bytes,
    ))
    .expect("test query runtime")
}

fn serve_once(response: Vec<u8>) -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test HTTP server");
    let address = listener.local_addr().expect("test HTTP server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept test HTTP request");
        let mut request = [0_u8; 1_024];
        let bytes_read = stream.read(&mut request).expect("read test HTTP request");
        assert!(bytes_read > 0, "test HTTP request must not be empty");
        stream
            .write_all(&response)
            .expect("write test HTTP response");
    });
    TestServer {
        url: Url::parse(&format!("http://{address}/data")).expect("test HTTP URL"),
        handle,
    }
}

fn content_length_response(declared_length: usize, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n"
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

fn chunked_response(chunks: &[&[u8]]) -> Vec<u8> {
    let mut response =
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
    for chunk in chunks {
        response.extend_from_slice(format!("{:X}\r\n", chunk.len()).as_bytes());
        response.extend_from_slice(chunk);
        response.extend_from_slice(b"\r\n");
    }
    response.extend_from_slice(b"0\r\n\r\n");
    response
}

fn redirect_response(location: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .into_bytes()
}
