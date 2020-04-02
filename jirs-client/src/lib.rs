use seed::fetch::FetchObject;
use seed::{prelude::*, *};

use jirs_data::IssueStatus;

use crate::model::Page;
use crate::shared::styled_select::StyledSelectChange;

mod api;
mod api_handlers;
mod login;
mod model;
mod project;
mod project_settings;
mod register;
mod shared;

pub type UserId = i32;
pub type IssueId = i32;
pub type AvatarFilterActive = bool;

#[derive(Clone, Debug)]
pub enum FieldId {
    IssueTypeEditModalTop,
}

#[derive(Clone, Debug)]
pub enum Msg {
    NoOp,
    StyledSelectChanged(FieldId, StyledSelectChange),

    ChangePage(model::Page),
    CurrentProjectResult(FetchObject<String>),
    CurrentUserResult(FetchObject<String>),
    InternalFailure(String),
    ToggleAboutTooltip,

    // project
    ProjectTextFilterChanged(String),
    ProjectAvatarFilterChanged(UserId, AvatarFilterActive),
    ProjectToggleOnlyMy,
    ProjectToggleRecentlyUpdated,
    ProjectClearFilters,

    // dragging
    IssueDragStarted(IssueId),
    IssueDragStopped(IssueId),
    IssueDropZone(IssueStatus),

    // issues
    IssueUpdateResult(FetchObject<String>),

    // modals
    CloseModal,
}

fn update(msg: Msg, model: &mut model::Model, orders: &mut impl Orders<Msg>) {
    if cfg!(debug_assertions) {
        log!(msg);
    }
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
        }
        Msg::CloseModal => {
            model.modal = None;
        }
        _ => (),
    }
    crate::shared::update(&msg, model, orders);
    match model.page {
        Page::Project => project::update(msg, model, orders),
        Page::EditIssue(_id) => project::update(msg, model, orders),
        Page::ProjectSettings => project_settings::update(msg, model, orders),
        Page::Login => login::update(msg, model, orders),
        Page::Register => register::update(msg, model, orders),
    }
    if cfg!(debug_assertions) {
        // debug!(model);
    }
}

fn view(model: &model::Model) -> Node<Msg> {
    match model.page {
        Page::Project => project::view(model),
        Page::EditIssue(_id) => project::view(model),
        Page::ProjectSettings => project_settings::view(model),
        Page::Login => login::view(model),
        Page::Register => register::view(model),
    }
}

fn routes(url: Url) -> Option<Msg> {
    if url.path.is_empty() {
        return Some(Msg::ChangePage(model::Page::Project));
    }

    match url.path[0].as_ref() {
        "board" => Some(Msg::ChangePage(model::Page::Project)),
        "issues" => match url.path.get(1).as_ref().map(|s| s.parse::<i32>()) {
            Some(Ok(id)) => Some(Msg::ChangePage(model::Page::EditIssue(id))),
            _ => None,
        },
        "project-settings" => Some(Msg::ChangePage(model::Page::ProjectSettings)),
        "login" => Some(Msg::ChangePage(model::Page::Login)),
        "register" => Some(Msg::ChangePage(model::Page::Register)),
        _ => Some(Msg::ChangePage(model::Page::Project)),
    }
}

pub static mut HOST_URL: String = String::new();

#[wasm_bindgen]
pub fn set_host_url(url: String) {
    unsafe {
        HOST_URL = url;
    }
}

#[wasm_bindgen]
pub fn render() {
    App::builder(update, view).routes(routes).build_and_start();
}
