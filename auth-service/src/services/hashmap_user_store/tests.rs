use super::*;
use crate::domain::{data_stores::UserStore, email};

async fn get_filled_hashmap_user_store() -> HashmapUserStore {
    let mut store = HashmapUserStore::default();
    let user = User::parse(
        "email@email.com".to_string(),
        "Password1!".to_string(),
        false,
    )
    .unwrap();
    store.add_user(user).await.unwrap();
    store
}

#[tokio::test]
async fn test_add_user() {
    let mut store = HashmapUserStore::default();

    let user = User::parse(
        "email@email.com".to_string(),
        "Password1!".to_string(),
        false,
    )
    .unwrap();
    store.add_user(user).await.unwrap();
    let email = Email {
        email: "email@email.com".to_string(),
    };
    assert!(store.users.contains_key(&email));
}

#[tokio::test]
async fn test_get_user() {
    let store = get_filled_hashmap_user_store().await;

    let user = store.get_user("email@email.com").await;
    assert!(user.is_ok());
    let user = user.unwrap();
    assert_eq!(user.email_str(), "email@email.com");
    assert_eq!(user.password_str(), "Password1!");
    assert!(!user.requires_2fa());

    let user = store.get_user("wrong_email@email.com").await;
    assert!(user.is_err());
    assert_eq!(user.unwrap_err(), UserStoreError::UserNotFound)
}

#[tokio::test]
async fn test_validate_user() {
    let store = get_filled_hashmap_user_store().await;
    let user = store.validate_user("email@email.com", "Password1!").await;
    assert!(user.is_ok());
    let user = store
        .validate_user("email@email.com", "wrong_password")
        .await;
    assert!(user.is_err());
    assert_eq!(user.unwrap_err(), UserStoreError::InvalidCredentials);
    let user = store
        .validate_user("wrong_email@email.com", "password")
        .await;
    assert!(user.is_err());
    assert_eq!(user.unwrap_err(), UserStoreError::UserNotFound);
}
