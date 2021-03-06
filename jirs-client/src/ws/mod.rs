use seed::prelude::*;

use jirs_data::WsMsg;

use crate::model::*;
use crate::shared::{go_to_board, write_auth_token};
use crate::{Msg, WebSocketChanged};

pub mod issue;

pub fn flush_queue(model: &mut Model, orders: &mut impl Orders<Msg>) {
    use seed::browser::web_socket::State;
    match model.ws.as_ref() {
        Some(ws) if ws.state() != State::Open => return,
        None => return,
        _ => (),
    };
    let mut old = vec![];
    std::mem::swap(&mut model.ws_queue, &mut old);
    for msg in old {
        send_ws_msg(msg, model.ws.as_ref(), orders);
    }
}

pub fn enqueue_ws_msg(v: Vec<WsMsg>, ws: Option<&WebSocket>, orders: &mut impl Orders<Msg>) {
    for msg in v {
        send_ws_msg(msg, ws, orders);
    }
}

pub fn send_ws_msg(msg: WsMsg, ws: Option<&WebSocket>, orders: &mut impl Orders<Msg>) {
    use seed::browser::web_socket::State;
    let ws = match ws {
        Some(ws) if ws.state() == State::Open => ws,
        _ => {
            orders
                .skip()
                .send_msg(Msg::WebSocketChange(WebSocketChanged::Bounced(msg)));
            return;
        }
    };
    let binary = bincode::serialize(&msg).unwrap();
    ws.send_bytes(binary.as_slice())
        .expect("Failed to send ws msg");
}

pub fn open_socket(model: &mut Model, orders: &mut impl Orders<Msg>) {
    use seed::browser::web_socket::State;
    use seed::{prelude::*, *};
    log!(model.ws.as_ref().map(|ws| ws.state()));

    match model.ws.as_ref() {
        Some(ws) if ws.state() != State::Closed => {
            return;
        }
        _ => (),
    };
    if model.host_url.is_empty() {
        return;
    }
    let url = model.ws_url.as_str();

    model.ws = WebSocket::builder(url, orders)
        .on_message(|msg| Msg::WebSocketChange(WebSocketChanged::WebSocketMessage(msg)))
        .on_open(|| Msg::WebSocketChange(WebSocketChanged::WebSocketOpened))
        .on_close(|_| Msg::WebSocketChange(WebSocketChanged::WebSocketClosed))
        .on_error(|| {})
        .build_and_open()
        .ok();
}

pub async fn read_incoming(msg: WebSocketMessage) -> Msg {
    let bytes = msg.bytes().await.unwrap_or_default();
    Msg::WebSocketChange(WebSocketChanged::WebSocketMessageLoaded(bytes))
}

pub fn update(msg: &WsMsg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        // auth
        WsMsg::AuthorizeLoaded(Ok(user)) => {
            model.user = Some(user.clone());
            if is_non_logged_area() {
                go_to_board(orders);
            }
            orders
                .skip()
                .send_msg(Msg::UserChanged(model.user.as_ref().cloned()));
        }
        WsMsg::AuthorizeExpired => {
            use seed::*;

            log!("Received token expired");
            if let Ok(msg) = write_auth_token(None) {
                orders.skip().send_msg(msg);
            }
        }
        // project
        WsMsg::ProjectsLoaded(v) => {
            model.projects = v.clone();
            init_current_project(model, orders);
        }
        // user projects
        WsMsg::UserProjectsLoaded(v) => {
            model.user_projects = v.clone();
            model.current_user_project = v.iter().find(|up| up.is_current).cloned();
            init_current_project(model, orders);
        }
        WsMsg::UserProjectCurrentChanged(user_project) => {
            let mut old = vec![];
            std::mem::swap(&mut old, &mut model.user_projects);
            for mut up in old {
                up.is_current = up.id == user_project.id;
                model.user_projects.push(up);
            }
            model.current_user_project = Some(user_project.clone());
            init_current_project(model, orders);
        }

        // issues
        WsMsg::ProjectIssuesLoaded(v) => {
            let mut v = v.clone();
            v.sort_by(|a, b| (a.list_position as i64).cmp(&(b.list_position as i64)));
            model.issues = v;
        }
        // issue statuses
        WsMsg::IssueStatusesResponse(v) => {
            model.issue_statuses = v.clone();
            model
                .issue_statuses
                .sort_by(|a, b| a.position.cmp(&b.position));
        }
        WsMsg::IssueStatusCreated(is) => {
            model.issue_statuses.push(is.clone());
            model
                .issue_statuses
                .sort_by(|a, b| a.position.cmp(&b.position));
        }
        WsMsg::IssueStatusUpdated(changed) => {
            let mut old = vec![];
            std::mem::swap(&mut model.issue_statuses, &mut old);
            for is in old {
                if is.id == changed.id {
                    model.issue_statuses.push(changed.clone());
                } else {
                    model.issue_statuses.push(is);
                }
            }
            model
                .issue_statuses
                .sort_by(|a, b| a.position.cmp(&b.position));
        }
        WsMsg::IssueStatusDeleted(dropped_id) => {
            let mut old = vec![];
            std::mem::swap(&mut model.issue_statuses, &mut old);
            for is in old {
                if is.id != *dropped_id {
                    model.issue_statuses.push(is);
                }
            }
            model
                .issue_statuses
                .sort_by(|a, b| a.position.cmp(&b.position));
        }
        WsMsg::IssueDeleted(id) => {
            let mut old = vec![];
            std::mem::swap(&mut model.issue_statuses, &mut old);
            for is in old {
                if is.id == *id {
                    continue;
                }
                model.issue_statuses.push(is);
            }
            model
                .issue_statuses
                .sort_by(|a, b| a.position.cmp(&b.position));
        }
        // users
        WsMsg::ProjectUsersLoaded(v) => {
            model.users = v.clone();
        }
        // comments
        WsMsg::IssueCommentsLoaded(comments) => {
            let issue_id = match model.modals.get(0) {
                Some(ModalType::EditIssue(issue_id, _)) => *issue_id,
                _ => return,
            };
            if comments.iter().any(|c| c.issue_id != issue_id) {
                return;
            }
            let mut v = comments.clone();
            v.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
            model.comments = v;
        }
        WsMsg::CommentDeleted(comment_id) => {
            let mut old = vec![];
            std::mem::swap(&mut model.comments, &mut old);
            for comment in old.into_iter() {
                if *comment_id != comment.id {
                    model.comments.push(comment);
                }
            }
        }
        WsMsg::AvatarUrlChanged(user_id, avatar_url) => {
            for user in model.users.iter_mut() {
                if user.id == *user_id {
                    user.avatar_url = Some(avatar_url.clone());
                }
            }
            if let Some(me) = model.user.as_mut() {
                if me.id == *user_id {
                    me.avatar_url = Some(avatar_url.clone());
                }
            }
        }
        // messages
        WsMsg::Message(received) => {
            let mut old = vec![];
            std::mem::swap(&mut old, &mut model.messages);
            for m in old {
                if m.id != received.id {
                    model.messages.push(m);
                } else {
                    model.messages.push(received.clone());
                }
            }
            model.messages.sort_by(|a, b| a.id.cmp(&b.id));
        }
        WsMsg::MessagesResponse(v) => {
            model.messages = v.clone();
            model.messages.sort_by(|a, b| a.id.cmp(&b.id));
        }
        WsMsg::MessageMarkedSeen(id) => {
            let mut old = vec![];
            std::mem::swap(&mut old, &mut model.messages);
            for m in old {
                if m.id != *id {
                    model.messages.push(m);
                }
            }
            model.messages.sort_by(|a, b| a.id.cmp(&b.id));
        }
        _ => (),
    };
    orders.render();
}

fn init_current_project(model: &mut Model, orders: &mut impl Orders<Msg>) {
    if model.projects.is_empty() {
        return;
    }
    model.project = model.current_user_project.as_ref().and_then(|up| {
        model
            .projects
            .iter()
            .find(|p| p.id == up.project_id)
            .cloned()
    });
    orders
        .skip()
        .send_msg(Msg::ProjectChanged(model.project.as_ref().cloned()));
}

fn is_non_logged_area() -> bool {
    let pathname = seed::document().location().unwrap().pathname().unwrap();
    match pathname.as_str() {
        "/login" | "/register" | "/invite" => true,
        _ => false,
    }
}
