use super::*;
use ic_host_process::testing::http::{HttpContentLength, capture_http_request};
use serde::Deserialize;
use std::{
    io::Write,
    net::TcpListener,
    thread::{self, JoinHandle},
    time::Duration,
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

#[tokio::test(flavor = "current_thread")]
async fn bounded_transport_accepts_valid_json_at_the_exact_byte_limit() {
    let body = br#"{"value":"ok"}"#;
    let server = serve_once(content_length_response(body.len(), body));

    let payload: TestPayload = fetch_test_json(&server.url, body.len() as u64)
        .await
        .expect("response at exact limit");
    server.finish();

    assert_eq!(
        payload,
        TestPayload {
            value: "ok".to_string()
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn bounded_transport_accepts_chunked_json_at_the_exact_byte_limit() {
    let body = br#"{"value":"ok"}"#;
    let server = serve_once(chunked_response(&[&body[..8], &body[8..]]));

    let payload: TestPayload = fetch_test_json(&server.url, body.len() as u64)
        .await
        .expect("chunked response at exact limit");
    server.finish();

    assert_eq!(payload.value, "ok");
}

#[tokio::test(flavor = "current_thread")]
async fn bounded_transport_preserves_status_before_body_and_size_failures() {
    let server = serve_once(
        b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 65\r\nConnection: close\r\n\r\n{"
            .to_vec(),
    );

    let error = fetch_test_json::<TestPayload>(&server.url, 0)
        .await
        .expect_err("unsuccessful status precedes size, truncation and JSON errors");
    server.finish();

    assert!(matches!(error, IcHostError::HttpStatus { status: 503, .. }));
}

#[tokio::test(flavor = "current_thread")]
async fn bounded_transport_zero_limit_admits_empty_bodies_before_json_decoding() {
    for response in [content_length_response(0, &[]), chunked_response(&[])] {
        let server = serve_once(response);
        let error = fetch_test_json::<TestPayload>(&server.url, 0)
            .await
            .expect_err("empty JSON remains a decoding error");
        server.finish();
        assert!(matches!(error, IcHostError::JsonDecode { .. }));
    }

    for response in [content_length_response(1, b"x"), chunked_response(&[b"x"])] {
        let server = serve_once(response);
        let error = fetch_test_json::<TestPayload>(&server.url, 0)
            .await
            .expect_err("a zero-byte budget refuses any response byte");
        server.finish();
        assert!(matches!(
            error,
            IcHostError::HttpResponseTooLarge {
                max_bytes: 0,
                observed_bytes: 1,
                ..
            }
        ));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bounded_transport_rejects_oversized_declared_length_before_reading_body() {
    let server = serve_once(content_length_response(65, &[]));

    let error = fetch_test_json::<TestPayload>(&server.url, 64)
        .await
        .expect_err("oversized declared response");
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

#[tokio::test(flavor = "current_thread")]
async fn bounded_transport_rejects_oversized_chunked_body_without_length_header() {
    let filler = vec![b'x'; 64];
    let server = serve_once(chunked_response(&[br#"{"value":""#, &filler, br#""}"#]));

    let error = fetch_test_json::<TestPayload>(&server.url, 32)
        .await
        .expect_err("oversized chunked response");
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

#[tokio::test(flavor = "current_thread")]
async fn bounded_transport_distinguishes_body_read_and_json_decode_failures() {
    let truncated = serve_once(content_length_response(100, b"{"));
    let read_error = fetch_test_json::<TestPayload>(&truncated.url, 200)
        .await
        .expect_err("truncated response body");
    truncated.finish();
    assert!(matches!(read_error, IcHostError::HttpResponseBody { .. }));

    let malformed = serve_once(content_length_response(8, b"not-json"));
    let decode_error = fetch_test_json::<TestPayload>(&malformed.url, 8)
        .await
        .expect_err("malformed response JSON");
    malformed.finish();
    assert!(matches!(decode_error, IcHostError::JsonDecode { .. }));
}

#[tokio::test(flavor = "current_thread")]
async fn dashboard_transport_rejects_redirects_without_following_the_location() {
    let server = serve_once(redirect_response("https://example.com/other-authority"));

    let error = fetch_test_json::<TestPayload>(&server.url, 100)
        .await
        .expect_err("redirect must be rejected");
    server.finish();

    assert!(matches!(error, IcHostError::HttpStatus { status: 302, .. }));
}

async fn fetch_test_json<T>(url: &Url, max_response_bytes: u64) -> Result<T, IcHostError>
where
    T: DeserializeOwned + Send,
{
    fetch_json_with_limit(http_client()?, url.clone(), max_response_bytes).await
}

fn serve_once(response: Vec<u8>) -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test HTTP server");
    let address = listener.local_addr().expect("test HTTP server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept test HTTP request");
        capture_http_request(
            &mut stream,
            Duration::from_secs(10),
            64 * 1024,
            0,
            HttpContentLength::Optional,
        )
        .into_result()
        .expect("read bounded test HTTP request");
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
