use super::*;

#[tokio::test]
async fn ban_token() {
    let mut store = HashSetBannedTokenStore::default();
    let token = "test.token";
    assert_eq!(store.tokens.len(), 0);
    store.ban_token(token.to_owned()).await;
    assert_eq!(store.tokens.len(), 1);
    assert!(store.tokens.contains(token));
}

#[tokio::test]
async fn is_valid() {
    let mut store = HashSetBannedTokenStore::default();
    let token = "test.token";
    store.ban_token(token.to_owned()).await;
    assert!(!store.is_valid(token).await);
    assert!(store.is_valid("something.else").await);
}
