// use actix_web::{http::StatusCode, test};

// mod helper;
// use helper::{create_user, post_json, spawn_app};

// #[actix_web::test]
// async fn test_create_event_success() {
//     let (app, _container) = spawn_app().await;

//     let token = create_user(&app).await;

//     let request = serde_json::json!({
//         "name": "RustConf",
//         "description": "A conference for Rust developers",
//         "capacity": 100,
//     });

//     let response =
//         test::call_service(&app, post_json("/api/v1/events/", &request, Some(&token))).await;

//     assert_eq!(response.status(), StatusCode::CREATED);

//     let body = test::read_body(response).await;

//     assert!(
//         uuid::Uuid::parse_str(&String::from_utf8(body.to_vec()).expect("response is UTF-8"))
//             .is_ok()
//     );
// }

// #[actix_web::test]
// async fn test_create_event_validation_fails() {
//     let (app, _container) = spawn_app().await;

//     let token = create_user(&app).await;

//     let request = serde_json::json!({
//         "name": "RustConf",
//         "capacity": "one hundred",
//     });

//     let response =
//         test::call_service(&app, post_json("/api/v1/events/", &request, Some(&token))).await;

//     assert_eq!(response.status(), StatusCode::BAD_REQUEST);
// }
