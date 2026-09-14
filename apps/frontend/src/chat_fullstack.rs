//! Typed UI transport over the existing owner-scoped chat adapter.
use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode},
};
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::chat::{hydrated::*, ChatConversation},
};
fn error(value: crate::chat_adapter::ChatLoadError) -> LoadError {
    match value {
        crate::chat_adapter::ChatLoadError::Forbidden => LoadError::Forbidden,
        crate::chat_adapter::ChatLoadError::Unavailable => LoadError::Unavailable,
        crate::chat_adapter::ChatLoadError::Malformed => LoadError::Malformed,
    }
}
pub async fn load(
    state: AppState,
    id: Option<uuid::Uuid>,
    headers: HeaderMap,
) -> Result<ChatData, LoadError> {
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let inbox = crate::chat_adapter::load_chat_inbox_for_ssr(
        state.wallet.as_ref(),
        &token,
        &user.wallet_address,
    )
    .await
    .map_err(error)?;
    let active = match id {
        Some(id) => Some(
            crate::chat_adapter::load_chat_detail_for_ssr(
                state.wallet.as_ref(),
                &token,
                &user.wallet_address,
                id,
            )
            .await
            .map_err(error)?,
        ),
        None => None,
    };
    Ok(ChatData { inbox, active })
}
fn request(headers: HeaderMap, body: Vec<u8>) -> Result<Request<Body>, LoadError> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/chat")
        .body(Body::from(body))
        .map_err(|_| LoadError::InvalidQuery)?;
    *request.headers_mut() = headers;
    Ok(request)
}
pub async fn mutate(
    state: AppState,
    command: ChatCommand,
    headers: HeaderMap,
) -> Result<ChatChanged, LoadError> {
    let (id,response)=match command{
        ChatCommand::Upload{id,filename,bytes}=>{
            if bytes.is_empty()||bytes.len()>5*1024*1024||filename.len()>255||filename.chars().any(|character|character.is_control()||matches!(character,'"'|'\\'|'/' )) {return Err(LoadError::InvalidQuery);}
            (Some(id),crate::chat_adapter::chat_upload_content(state,headers,id,filename,bytes).await)
        },
        ChatCommand::Create{topic_id,subject,message}=>(None,crate::chat_adapter::chat_create_api(State(state),request(headers,serde_json::to_vec(&serde_json::json!({"topic_id":topic_id,"subject":subject,"message":message})).map_err(|_|LoadError::InvalidQuery)?)?).await),
        ChatCommand::Send{id,content}=>(Some(id),crate::chat_adapter::chat_send_api(State(state),Path(id),request(headers,serde_json::to_vec(&serde_json::json!({"content":content})).map_err(|_|LoadError::InvalidQuery)?)?).await),
        ChatCommand::Resolve{id}=>{let response=crate::chat_adapter::chat_conversation_form(State(state),Path(id),request(headers,b"operation=resolve".to_vec())?).await;if response.headers().get("location").and_then(|value|value.to_str().ok())==Some(format!("/chat/{id}?chat=resolved").as_str()){return Ok(ChatChanged{conversation_id:id});}return Err(match response.status() { StatusCode::UNAUTHORIZED => LoadError::Unauthenticated, StatusCode::FORBIDDEN => LoadError::Forbidden, _ => LoadError::Unavailable });},
    };
    if !response.status().is_success() {
        return Err(match response.status() {
            StatusCode::UNAUTHORIZED => LoadError::Unauthenticated,
            StatusCode::FORBIDDEN => LoadError::Forbidden,
            StatusCode::BAD_REQUEST => LoadError::InvalidQuery,
            _ => LoadError::Unavailable,
        });
    }
    let body = axum::body::to_bytes(response.into_body(), 512 * 1024)
        .await
        .map_err(|_| LoadError::Malformed)?;
    let envelope: serde_json::Value =
        serde_json::from_slice(&body).map_err(|_| LoadError::Malformed)?;
    if envelope.get("success").and_then(|value| value.as_bool()) != Some(true) {
        return Err(LoadError::Malformed);
    }
    let id = match id {
        Some(id) => id,
        None => serde_json::from_value::<ChatConversation>(
            envelope.get("data").cloned().ok_or(LoadError::Malformed)?,
        )
        .map_err(|_| LoadError::Malformed)?
        .id
        .parse()
        .map_err(|_| LoadError::Malformed)?,
    };
    Ok(ChatChanged {
        conversation_id: id,
    })
}
