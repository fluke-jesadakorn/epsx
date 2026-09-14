use super::*;
use crate::fullstack::LoadError;
use crate::pages::analytics::WatchlistData;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WatchlistCommand {
    Save {
        symbol: String,
        group_ids: Vec<Uuid>,
    },
    Remove {
        symbol: String,
    },
    CreateGroup {
        name: String,
    },
    RenameGroup {
        id: Uuid,
        name: String,
    },
    DeleteGroup {
        id: Uuid,
    },
    Layout(WatchlistLayoutUpdate),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WatchlistChange {
    Symbols(WatchlistData),
    Layout(WatchlistLayoutData),
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct WatchlistProvider(pub WatchlistProviderCallback);
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct WatchlistMutationProvider(pub WatchlistMutationProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "watchlist")]
pub async fn read_watchlist() -> Result<Result<WatchlistLayoutData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<WatchlistProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Watchlist provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(headers).await)
}
#[server(prefix = "/_server/frontend", endpoint = "watchlist-change")]
pub async fn change_watchlist(
    command: WatchlistCommand,
) -> Result<Result<WatchlistChange, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<WatchlistMutationProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Watchlist provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(command, headers).await)
}

/// Small reusable Dioxus mutation control for ranking and stock cards.
#[component]
pub fn WatchButton(
    symbol: String,
    initially_saved: bool,
    #[props(default)] in_place: bool,
    #[props(default)] class: String,
    #[props(default)] heart: bool,
) -> Element {
    let mut saved = use_signal(|| initially_saved);
    let mut pending = use_signal(|| false);
    let mut error = use_signal(String::new);
    let label = if saved() { "Saved" } else { "Save" };
    let target = symbol.clone();
    rsx! {
        button { class:class,r#type:"button",disabled:pending(),aria_pressed:saved(),aria_busy:pending(),aria_label:if saved(){format!("Saved · Remove {symbol} from saved companies")}else{format!("Save {symbol}")},
            "data-watchlist-in-place":in_place.then_some("true"),
            onclick:move |_|{if *pending.peek(){return;}pending.set(true);error.set(String::new());let symbol=target.clone();let command=if saved(){WatchlistCommand::Remove{symbol:symbol.clone()}}else{WatchlistCommand::Save{symbol:symbol.clone(),group_ids:vec![]}};
                spawn(async move{match change_watchlist(command).await.unwrap_or(Err(LoadError::Unavailable)){
                    Ok(WatchlistChange::Symbols(data))=>saved.set(data.symbols.iter().any(|candidate|candidate==&symbol)),
                    Ok(_)=>error.set(LoadError::Malformed.message().into()),Err(failure)=>error.set(failure.message().into()),
                }pending.set(false);});},
            if heart { span { aria_hidden: "true", if saved() { "♥" } else { "♡" } } } else { Icon{name:"bookmark".to_string(),size:Some(17)}span{"{label}"} }
        }
        if !error().is_empty(){span{role:"status",class:"text-xs fe-tone-danger",crate::fullstack::load_error::SessionMessage{message:error()}}}
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Organize {
    MoveTo(String, Option<Uuid>, Option<Uuid>),
    GroupMove(Uuid, isize),
    ItemMove(String, Option<Uuid>, isize),
    Membership(String, Vec<Uuid>),
    RemoveMembership(String, Uuid),
}
fn updated_layout(layout: &WatchlistLayoutData, action: Organize) -> WatchlistLayoutUpdate {
    let mut update = WatchlistLayoutUpdate {
        groups: layout
            .groups
            .iter()
            .map(|group| WatchlistGroupLayoutUpdate {
                id: group.id,
                symbols: group.symbols.clone(),
            })
            .collect(),
        ungrouped: layout.ungrouped.clone(),
    };
    match action {
        Organize::MoveTo(symbol, source, destination) => {
            match source {
                Some(id) => {
                    if let Some(group) = update.groups.iter_mut().find(|group| group.id == id) {
                        group.symbols.retain(|value| value != &symbol);
                    }
                }
                None => update.ungrouped.retain(|value| value != &symbol),
            }
            match destination {
                Some(id) => {
                    if let Some(group) = update.groups.iter_mut().find(|group| group.id == id) {
                        if !group.symbols.contains(&symbol) {
                            group.symbols.push(symbol);
                        }
                    }
                }
                None => {
                    for group in &mut update.groups {
                        group.symbols.retain(|value| value != &symbol);
                    }
                    if !update.ungrouped.contains(&symbol) {
                        update.ungrouped.push(symbol);
                    }
                }
            }
        }
        Organize::GroupMove(id, delta) => {
            if let Some(index) = update.groups.iter().position(|group| group.id == id) {
                let target = index
                    .saturating_add_signed(delta)
                    .min(update.groups.len() - 1);
                update.groups.swap(index, target);
            }
        }
        Organize::ItemMove(symbol, group, delta) => {
            let symbols = match group {
                Some(id) => update
                    .groups
                    .iter_mut()
                    .find(|group| group.id == id)
                    .map(|group| &mut group.symbols),
                None => Some(&mut update.ungrouped),
            };
            if let Some(symbols) = symbols {
                if let Some(index) = symbols.iter().position(|value| value == &symbol) {
                    let target = index.saturating_add_signed(delta).min(symbols.len() - 1);
                    symbols.swap(index, target);
                }
            }
        }
        Organize::Membership(symbol, ids) => {
            for group in &mut update.groups {
                if ids.contains(&group.id) {
                    if !group.symbols.contains(&symbol) {
                        group.symbols.push(symbol.clone());
                    }
                } else {
                    group.symbols.retain(|value| value != &symbol);
                }
            }
            update.ungrouped.retain(|value| value != &symbol);
            if ids.is_empty() {
                update.ungrouped.push(symbol);
            }
        }
        Organize::RemoveMembership(symbol, id) => {
            for group in &mut update.groups {
                if group.id == id {
                    group.symbols.retain(|value| value != &symbol);
                }
            }
            if !update
                .groups
                .iter()
                .any(|group| group.symbols.contains(&symbol))
                && !update.ungrouped.contains(&symbol)
            {
                update.ungrouped.push(symbol);
            }
        }
    }
    update
}
#[derive(Clone, Debug)]
pub(super) enum Dragged {
    Group(Uuid),
    Item(String, Option<Uuid>),
}
#[derive(Clone, Copy)]
pub(super) struct PortfolioControls {
    pub change: EventHandler<WatchlistCommand>,
    pub organize: EventHandler<Organize>,
    pub pending: ReadSignal<bool>,
    pub completed: ReadSignal<u64>,
    pub dragged: Signal<Option<Dragged>>,
    pub keyboard: EventHandler<(Dragged, Key)>,
}

fn requires_sign_in(error: Option<&LoadError>) -> bool {
    matches!(error, Some(LoadError::Unauthenticated))
}

#[component]
pub fn HydratedPortfolio() -> Element {
    let initial = use_server_future(|| async {
        read_watchlist()
            .await
            .unwrap_or(Err(LoadError::Unavailable))
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    let mut layout = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut completed = use_signal(|| 0_u64);
    let mut dragged = use_signal(|| None::<Dragged>);
    let mut preview = use_signal(|| None::<WatchlistLayoutData>);
    let mut announcement = use_signal(String::new);
    let change = use_callback(move |command: WatchlistCommand| {
        if *pending.peek() {
            return;
        }
        pending.set(true);
        error.set(None);
        spawn(async move {
            let result = match change_watchlist(command)
                .await
                .unwrap_or(Err(LoadError::Unavailable))
            {
                Ok(WatchlistChange::Layout(value)) => Ok(value),
                Ok(WatchlistChange::Symbols(_)) => read_watchlist()
                    .await
                    .unwrap_or(Err(LoadError::Unavailable)),
                Err(failure) => Err(failure),
            };
            match result {
                Ok(value) => {
                    layout.set(Some(value));
                    let next = *completed.peek() + 1;
                    completed.set(next);
                }
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        layout.set(None);
                    }
                    error.set(Some(failure));
                }
            }
            pending.set(false);
        });
    });
    let organize = use_callback(move |action: Organize| {
        if let Some(layout) = layout.peek().as_ref() {
            change.call(WatchlistCommand::Layout(updated_layout(layout, action)));
        }
    });
    let keyboard = use_callback(move |(item, key): (Dragged, Key)| {
        let commit = key == Key::Enter || key == Key::Character(" ".into());
        if key == Key::Escape {
            dragged.set(None);
            preview.set(None);
            announcement.set("Move cancelled.".into());
            return;
        }
        if commit {
            if let Some(value) = preview() {
                change.call(WatchlistCommand::Layout(WatchlistLayoutUpdate {
                    groups: value
                        .groups
                        .into_iter()
                        .map(|group| WatchlistGroupLayoutUpdate {
                            id: group.id,
                            symbols: group.symbols,
                        })
                        .collect(),
                    ungrouped: value.ungrouped,
                }));
                dragged.set(None);
                preview.set(None);
                announcement.set("Move submitted.".into());
            } else {
                dragged.set(Some(item));
                preview.set(layout());
                announcement.set("Picked up. Use arrows to move, Tab to change groups, Enter to save, Escape to cancel.".into());
            }
            return;
        }
        let Some(mut value) = preview() else {
            return;
        };
        let Some(active) = dragged() else {
            return;
        };
        let action = match active {
            Dragged::Group(id) => {
                let delta = match key {
                    Key::ArrowUp | Key::ArrowLeft => -1,
                    Key::ArrowDown | Key::ArrowRight => 1,
                    _ => return,
                };
                Organize::GroupMove(id, delta)
            }
            Dragged::Item(symbol, group) => {
                if key == Key::Tab {
                    let groups = std::iter::once(None)
                        .chain(value.groups.iter().map(|group| Some(group.id)))
                        .collect::<Vec<_>>();
                    let index = groups.iter().position(|id| id == &group).unwrap_or(0);
                    let destination = groups[(index + 1) % groups.len()];
                    dragged.set(Some(Dragged::Item(symbol.clone(), destination)));
                    Organize::MoveTo(symbol, group, destination)
                } else {
                    let delta = match key {
                        Key::ArrowUp | Key::ArrowLeft => -1,
                        Key::ArrowDown | Key::ArrowRight => 1,
                        _ => return,
                    };
                    Organize::ItemMove(symbol, group, delta)
                }
            }
        };
        let update = updated_layout(&value, action);
        let old = value.groups.clone();
        value.groups = update
            .groups
            .into_iter()
            .enumerate()
            .filter_map(|(position, group)| {
                old.iter()
                    .find(|old| old.id == group.id)
                    .map(|old| WatchlistGroupData {
                        id: group.id,
                        name: old.name.clone(),
                        position: position as i32,
                        symbols: group.symbols,
                    })
            })
            .collect();
        value.ungrouped = update.ungrouped;
        preview.set(Some(value));
        announcement.set("Position changed. Press Enter to save or Escape to cancel.".into());
    });
    use_context_provider(|| PortfolioControls {
        change,
        organize,
        pending: pending.into(),
        completed: completed.into(),
        dragged,
        keyboard,
    });
    let current_error = error();
    let signed_out = requires_sign_in(current_error.as_ref());
    rsx! {
        document::Title{"Saved companies — EPSX"}document::Meta{name:"description",content:"Organize and sync your saved companies."}
        div{class:"fe-saved-page portfolio-prod-container",aria_busy:pending(),
            PortfolioHeader { freshness: if layout().is_some() { "ready" } else if signed_out { "signed_out" } else { "unavailable" }, watched_count: layout().map(|value|value.watched).unwrap_or_default() }
            if signed_out {
                PortfolioSignInCard {}
            } else {
                if !announcement().is_empty(){p{role:"status",aria_live:"polite","{announcement}"}}
                if let Some(failure)=error(){crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(), button{class:"btn btn-outline",disabled:pending(),onclick:move |_|{pending.set(true);spawn(async move{match read_watchlist().await.unwrap_or(Err(LoadError::Unavailable)){Ok(value)=>{layout.set(Some(value));error.set(None);},Err(failure)=>error.set(Some(failure))}pending.set(false);});},"Try again"} }}
                if let Some(layout)=layout(){PortfolioWatchlist{layout}}
            }
        }
    }
}

#[cfg(feature = "server")]
pub type WatchlistProviderCallback = std::sync::Arc<
    dyn Fn(
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<WatchlistLayoutData, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type WatchlistMutationProviderCallback = std::sync::Arc<
    dyn Fn(
            WatchlistCommand,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<WatchlistChange, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unauthenticated_watchlist_load_uses_the_sign_in_state() {
        assert!(requires_sign_in(Some(&LoadError::Unauthenticated)));
        assert!(!requires_sign_in(Some(&LoadError::Unavailable)));
        assert!(!requires_sign_in(None));
    }

    #[test]
    fn removing_last_membership_preserves_saved_company() {
        let id = Uuid::nil();
        let layout = WatchlistLayoutData {
            groups: vec![WatchlistGroupData {
                id,
                name: "Long term".into(),
                position: 0,
                symbols: vec!["AAPL".into()],
            }],
            ungrouped: vec![],
            watched: 1,
        };
        let result = updated_layout(&layout, Organize::RemoveMembership("AAPL".into(), id));
        assert!(result.groups[0].symbols.is_empty());
        assert_eq!(result.ungrouped, vec!["AAPL"]);
    }
    #[test]
    fn moving_first_item_up_does_not_wrap() {
        let layout = WatchlistLayoutData {
            groups: vec![],
            ungrouped: vec!["AAPL".into(), "MSFT".into()],
            watched: 2,
        };
        let result = updated_layout(&layout, Organize::ItemMove("AAPL".into(), None, -1));
        assert_eq!(result.ungrouped, layout.ungrouped);
    }
}
