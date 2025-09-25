use uuid::Uuid;

use super::*;

#[tokio::test]
async fn test_add_code() {
    let mut store = HashMapTwoFACodeStore::default();

    let email = Email::parse("email@email.com").unwrap();
    let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string()).unwrap();
    let code = TwoFACode::default();
    store
        .add_code(email.clone(), login_attempt_id.clone(), code.clone())
        .await
        .unwrap();
    assert_eq!(store.codes.len(), 1);
    assert!(store.codes.contains_key(&email));
    assert_eq!(
        store.codes.get(&email),
        Some((login_attempt_id, code)).as_ref()
    )
}

#[tokio::test]
async fn test_add_code_fail() {
    let mut store = HashMapTwoFACodeStore::default();

    let email = Email::parse("email@email.com").unwrap();
    let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string()).unwrap();
    let code = TwoFACode::default();
    store
        .add_code(email.clone(), login_attempt_id.clone(), code.clone())
        .await
        .unwrap();

    let result = store
        .add_code(email.clone(), login_attempt_id.clone(), code.clone())
        .await;

    assert!(result.is_err());
    assert_eq!(
        result.err(),
        Some(TwoFACodeStoreError::UnexpectedError(
            HashMapTwoFACodeStoreError::InsertError.into()
        ))
    )
}

#[tokio::test]
async fn test_remove_code() {
    let mut store = HashMapTwoFACodeStore::default();

    let email = Email::parse("email@email.com").unwrap();
    let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string()).unwrap();
    let code = TwoFACode::default();
    store
        .add_code(email.clone(), login_attempt_id.clone(), code.clone())
        .await
        .unwrap();
    assert_eq!(store.codes.len(), 1);
    store.remove_code(&email).await.unwrap();
    assert_eq!(store.codes.len(), 0);
}

#[tokio::test]
async fn test_remove_fail() {
    let mut store = HashMapTwoFACodeStore::default();

    let email = Email::parse("email@email.com").unwrap();
    let result = store.remove_code(&email).await;
    assert!(result.is_err());
    assert_eq!(
        result.err(),
        Some(TwoFACodeStoreError::UnexpectedError(
            HashMapTwoFACodeStoreError::RemoveError.into()
        ))
    );
}

#[tokio::test]
async fn test_get_code() {
    let mut store = HashMapTwoFACodeStore::default();

    let email = Email::parse("email@email.com").unwrap();
    let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string()).unwrap();
    let code = TwoFACode::default();
    store
        .add_code(email.clone(), login_attempt_id.clone(), code.clone())
        .await
        .unwrap();
    let value = store.get_code(&email).await.unwrap();
    assert_eq!(value.0, login_attempt_id);
    assert_eq!(value.1, code);
    assert_eq!(store.codes.len(), 1);
    assert!(store.codes.contains_key(&email));
}

#[tokio::test]
async fn test_get_code_fail() {
    let store = HashMapTwoFACodeStore::default();

    let email = Email::parse("email@email.com").unwrap();
    let result = store.get_code(&email).await;
    assert!(result.is_err());
    assert_eq!(
        result.err(),
        Some(TwoFACodeStoreError::UnexpectedError(
            HashMapTwoFACodeStoreError::GetError.into()
        ))
    );
}
