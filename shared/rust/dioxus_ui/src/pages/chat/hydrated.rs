use super::*;
use crate::fullstack::LoadError;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatData {
    pub inbox: ChatInboxData,
    pub active: Option<ChatDetailData>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChatCommand {
    Upload {
        id: Uuid,
        filename: String,
        bytes: Vec<u8>,
    },
    Create {
        topic_id: String,
        subject: String,
        message: String,
    },
    Send {
        id: Uuid,
        content: String,
    },
    Resolve {
        id: Uuid,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatChanged {
    pub conversation_id: Uuid,
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct ChatProvider(pub ChatProviderCallback);
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct ChatMutationProvider(pub ChatMutationProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "chat")]
pub async fn read_chat(id: Option<Uuid>) -> Result<Result<ChatData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<ChatProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Chat provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(id, headers).await)
}
#[server(prefix = "/_server/frontend", endpoint = "chat-change")]
pub async fn change_chat(
    command: ChatCommand,
) -> Result<Result<ChatChanged, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<ChatMutationProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Chat provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(command, headers).await)
}
#[derive(Clone, Copy)]
pub(super) struct ChatControls {
    pub mutate: EventHandler<ChatCommand>,
    pub pending: ReadSignal<bool>,
    pub sent: ReadSignal<u64>,
}
#[component]
pub fn HydratedChat(
    id: ReadSignal<Option<Uuid>>,
    query: ReadSignal<String>,
    #[props(default)] history: bool,
) -> Element {
    let initial_id = use_hook(|| *id.read());
    let initial = use_server_future(move || async move {
        read_chat(initial_id)
            .await
            .unwrap_or(Err(LoadError::Unavailable))
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut revision = use_signal(|| 0u64);
    let mut generation = use_signal(|| 0u64);
    let mut last_id = use_signal(|| initial_id);
    let mut sent = use_signal(|| 0_u64);
    let navigation = use_navigator();
    let mutate = use_callback(move |command: ChatCommand| {
        if *pending.peek() {
            return;
        }
        pending.set(true);
        error.set(None);
        let is_send = matches!(&command, ChatCommand::Send { .. });
        spawn(async move {
            match change_chat(command)
                .await
                .unwrap_or(Err(LoadError::Unavailable))
            {
                Ok(value) => {
                    if is_send {
                        let next = *sent.peek() + 1;
                        sent.set(next);
                    }
                    navigation.push(format!("/chat/{}", value.conversation_id));
                    let next = *revision.peek() + 1;
                    revision.set(next);
                }
                Err(failure) => error.set(Some(failure)),
            }
            pending.set(false);
        });
    });
    use_context_provider(|| ChatControls {
        mutate,
        pending: pending.into(),
        sent: sent.into(),
    });
    use_effect(move || {
        let id = id();
        let revision = revision();
        if id == *last_id.peek() && revision == 0 {
            return;
        }
        if id != *last_id.peek() {
            if let Some(value) = data.write().as_mut() {
                value.active = None;
            }
        }
        pending.set(true);
        last_id.set(id);
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        spawn(async move {
            let result = read_chat(id).await.unwrap_or(Err(LoadError::Unavailable));
            if *generation.peek() != ticket {
                return;
            }
            match result {
                Ok(value) => {
                    data.set(Some(value));
                    error.set(None);
                }
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(None);
                    }
                    error.set(Some(failure));
                }
            }
            pending.set(false);
        });
    });
    rsx! {
        document::Title{"Support — EPSX"}document::Meta{name:"description",content:"Your private support conversations with EPSX."}
        if error() == Some(LoadError::Unauthenticated) { RenderPublicChat {} }
        else if let Some(failure)=error(){p{role:"status","{failure.message()}"}button{class:"btn btn-outline",disabled:pending(),onclick:move |_|{let next=*revision.peek()+1;revision.set(next);},"Try again"}}
        if let Some(data)=data(){ChatReady{inbox:data.inbox,active:data.active,show_new:query().split('&').any(|part|part=="new=1"),history}}
    }
}

#[component]
pub(super) fn NewConversation(topics: Vec<ChatTopic>) -> Element {
    let mut selected = use_signal(|| None::<ChatTopic>);
    let mut subject = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut attachment = use_signal(|| None::<(String, Vec<u8>)>);
    let mut pending = use_signal(|| false);
    let mut status = use_signal(String::new);
    let mut created = use_signal(|| None::<Uuid>);
    let navigation = use_navigator();
    let choose_file = move |event: FormEvent| {
        if let Some(file) = event.files().first().cloned() {
            pending.set(true);
            spawn(async move {
                match file.read_bytes().await {
                    Ok(bytes) if !bytes.is_empty() && bytes.len() <= 5 * 1024 * 1024 => {
                        attachment.set(Some((file.name(), bytes.to_vec())));
                        status.set(String::new());
                    }
                    _ => status.set("Choose a file up to 5MB.".into()),
                }
                pending.set(false);
            });
        }
    };
    let submit = move |event: FormEvent| {
        event.prevent_default();
        if *pending.peek() {
            return;
        }
        let Some(topic) = selected() else {
            return;
        };
        pending.set(true);
        status.set(String::new());
        spawn(async move {
            let id = match created() {
                Some(id) => id,
                None => match change_chat(ChatCommand::Create {
                    topic_id: topic.id,
                    subject: subject(),
                    message: message(),
                })
                .await
                .unwrap_or(Err(LoadError::Unavailable))
                {
                    Ok(value) => {
                        created.set(Some(value.conversation_id));
                        value.conversation_id
                    }
                    Err(error) => {
                        status.set(error.message().into());
                        pending.set(false);
                        return;
                    }
                },
            };
            if let Some((filename, bytes)) = attachment() {
                if let Err(error) = change_chat(ChatCommand::Upload {
                    id,
                    filename,
                    bytes,
                })
                .await
                .unwrap_or(Err(LoadError::Unavailable))
                {
                    status.set(format!("Conversation created. Attachment failed: {} Retry sends only the attachment.",error.message()));
                    pending.set(false);
                    return;
                }
            }
            navigation.push(format!("/chat/{id}"));
            pending.set(false);
        });
    };
    rsx! {
        section{class:"chat-panel chat-panel-new","data-chat-surface":"new-conversation",
            if let Some(topic)=selected(){
                div{class:"chat-topic-form-wrap",
                    button{class:"chat-topic-back",r#type:"button",disabled:pending()||created().is_some(),onclick:move |_|selected.set(None),Icon{name:"arrow-left".to_string(),size:Some(14)},"Back to topics"}
                    div{class:"chat-topic-header",div{class:"chat-topic-icon",Icon{name:topic.icon.clone().unwrap_or_else(||"message-circle".into()),size:Some(20)}}div{p{class:"chat-topic-label","{topic.label}"}p{class:"chat-topic-description","{topic.description.as_deref().unwrap_or_default()}"}}}
                    form{class:"chat-topic-composer",onsubmit:submit,
                        label{class:"chat-topic-form-label",r#for:"chat-subject","SUBJECT"}
                        input{class:"chat-topic-form-input",id:"chat-subject",required:true,maxlength:"255",placeholder:"Brief summary of your issue",disabled:pending()||created().is_some(),value:subject(),oninput:move |event|subject.set(event.value())}
                        label{class:"chat-topic-form-label",r#for:"chat-message","MESSAGE"}
                        textarea{class:"chat-topic-form-textarea",id:"chat-message",required:true,maxlength:"16384",placeholder:"Describe your issue in detail...",disabled:pending()||created().is_some(),value:message(),oninput:move |event|message.set(event.value())}
                        label{class:"chat-topic-dropzone",Icon{name:"paperclip".to_string(),size:Some(18)}p{"Attach a screenshot or file"}p{class:"chat-topic-dropzone-hint","JPG, PNG, GIF, WebP, PDF · Max 5MB"}
                            input{r#type:"file",accept:".jpg,.jpeg,.png,.gif,.webp,.pdf",disabled:pending(),onchange:choose_file}
                            if let Some((name,_))=attachment(){p{class:"chat-topic-file-list","{name}"}}
                        }
                        if !status().is_empty(){p{role:"status",class:"chat-topic-form-status","{status}"}}
                        button{class:"chat-topic-start",r#type:"submit",disabled:pending()||subject().trim().is_empty()||message().trim().is_empty(),if pending(){"Sending…"}else if created().is_some(){"Retry attachment"}else{"Start conversation"}}
                        if let Some(id)=created(){crate::fullstack::shell::ShellLink{href:format!("/chat/{id}"),"Open created conversation"}}
                    }
                }
            }else{
                div{class:"chat-topic-selector",h2{class:"chat-panel-title","How can we help?"}p{class:"chat-panel-description","Choose a topic to start a conversation."}
                    div{class:"chat-topic-grid",for topic in topics{button{class:"chat-topic-card",r#type:"button",onclick:move |_|selected.set(Some(topic.clone())),h3{"{topic.label}"}p{"{topic.description.as_deref().unwrap_or_default()}"}}}}
                }
            }
        }
    }
}

#[cfg(feature = "server")]
pub type ChatProviderCallback = std::sync::Arc<
    dyn Fn(
            Option<Uuid>,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<ChatData, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type ChatMutationProviderCallback = std::sync::Arc<
    dyn Fn(
            ChatCommand,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<ChatChanged, LoadError>> + Send>,
        > + Send
        + Sync,
>;
