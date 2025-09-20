use super::*;

#[tokio::test]
async fn ban_token() {
    let mut store = HashSetBannedTokenStore::default();
    let token = "test.token";
    assert_eq!(store.tokens.len(), 0);
    store.add_token(token.to_owned()).await.unwrap();
    assert_eq!(store.tokens.len(), 1);
    assert!(store.tokens.contains(token));
}

#[tokio::test]
async fn is_valid() {
    let mut store = HashSetBannedTokenStore::default();
    let token = "test.token";
    store.add_token(token.to_owned()).await.unwrap();
    assert!(store.contains_token(token).await.unwrap());
    assert!(!store.contains_token("something.else").await.unwrap());
}
