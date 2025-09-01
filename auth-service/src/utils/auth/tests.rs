use super::*;

#[tokio::test]
async fn test_generate_auth_cookie() {
    let email = Email::parse("test@example.com").unwrap();
    let cookie = generate_auth_cookie(&email).unwrap();
    assert_eq!(cookie.name(), JWT_COOKIE_NAME);
    assert_eq!(cookie.value().split('.').count(), 3);
    assert_eq!(cookie.path(), Some("/"));
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
}

#[tokio::test]
async fn test_create_auth_cookie() {
    let token = "test_token".to_owned();
    let cookie = create_auth_cookie(token.clone());
    assert_eq!(cookie.name(), JWT_COOKIE_NAME);
    assert_eq!(cookie.value(), token);
    assert_eq!(cookie.path(), Some("/"));
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
}

#[tokio::test]
async fn test_generate_auth_token() {
    let email = Email::parse("test@example.com").unwrap();
    let result = generate_auth_token(&email).unwrap();
    assert_eq!(result.split('.').count(), 3);
}

#[tokio::test]
async fn test_validate_token_with_valid_token() {
    let email = Email::parse("test@example.com").unwrap();
    let token = generate_auth_token(&email).unwrap();
    let result = validate_token(&token).await.unwrap();
    assert_eq!(result.sub, "test@example.com");

    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
        .expect("valid timestamp")
        .timestamp();

    assert!(result.exp > exp as usize);
}

#[tokio::test]
async fn test_validate_token_with_invalid_token() {
    let token = "invalid_token".to_owned();
    let result = validate_token(&token).await;
    assert!(result.is_err());
}
