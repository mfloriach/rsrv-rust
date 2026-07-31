use actix_web::http::header::ContentType;
use actix_web::test;

pub fn post_json<T: serde::Serialize>(uri: &str, body: &T) -> actix_http::Request {
    test::TestRequest::post()
        .uri(uri)
        .insert_header(ContentType::json())
        .set_json(body)
        .to_request()
}
