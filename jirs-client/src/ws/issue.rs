use seed::prelude::Orders;
use seed::*;

use jirs_data::*;

use crate::model::{Model, PageContent};
use crate::ws::send_ws_msg;
use crate::Msg;

pub fn drag_started(issue_id: IssueId, model: &mut Model) {
    let project_page = match &mut model.page_content {
        PageContent::Project(project_page) => project_page,
        _ => return,
    };
    project_page.issue_drag.drag(issue_id);
}

pub fn exchange_position(issue_bellow_id: IssueId, model: &mut Model) {
    let project_page = match &mut model.page_content {
        PageContent::Project(project_page) => project_page,
        _ => return,
    };
    if project_page.issue_drag.dragged_or_last(issue_bellow_id) {
        return;
    }
    let dragged_id = match project_page.issue_drag.dragged_id.as_ref().cloned() {
        Some(id) => id,
        _ => return error!("Nothing is dragged"),
    };

    let mut below = None;
    let mut dragged = None;
    let mut issues = vec![];
    std::mem::swap(&mut issues, &mut model.issues);

    for issue in issues.into_iter() {
        match issue.id {
            id if id == issue_bellow_id => below = Some(issue),
            id if id == dragged_id => dragged = Some(issue),
            _ => model.issues.push(issue),
        };
    }

    let mut below = match below {
        Some(below) => below,
        _ => return,
    };
    let mut dragged = match dragged {
        Some(issue) => issue,
        _ => {
            model.issues.push(below);
            return;
        }
    };
    if dragged.issue_status_id != below.issue_status_id {
        let mut issues = vec![];
        std::mem::swap(&mut issues, &mut model.issues);
        for mut c in issues.into_iter() {
            if c.issue_status_id == below.issue_status_id && c.list_position > below.list_position {
                c.list_position += 1;
                project_page.issue_drag.mark_dirty(c.id);
            }
            model.issues.push(c);
        }
        dragged.list_position = below.list_position + 1;
        dragged.issue_status_id = below.issue_status_id;
    }
    std::mem::swap(&mut dragged.list_position, &mut below.list_position);

    project_page.issue_drag.mark_dirty(dragged.id);
    project_page.issue_drag.mark_dirty(below.id);

    model.issues.push(below);
    model.issues.push(dragged);
    model
        .issues
        .sort_by(|a, b| a.list_position.cmp(&b.list_position));
    project_page.issue_drag.last_id = Some(issue_bellow_id);
}

pub fn sync(model: &mut Model, orders: &mut impl Orders<Msg>) {
    // log!("------------------------------------------------------------------");
    // log!("|                SYNC                                            |");
    // log!("------------------------------------------------------------------");
    let project_page = match &mut model.page_content {
        PageContent::Project(project_page) => project_page,
        _ => return,
    };

    for issue in model.issues.iter() {
        if !project_page.issue_drag.dirty.contains(&issue.id) {
            continue;
        }

        send_ws_msg(
            WsMsg::IssueUpdateRequest(
                issue.id,
                IssueFieldId::IssueStatusId,
                PayloadVariant::I32(issue.issue_status_id),
            ),
            model.ws.as_ref(),
            orders,
        );
        send_ws_msg(
            WsMsg::IssueUpdateRequest(
                issue.id,
                IssueFieldId::ListPosition,
                PayloadVariant::I32(issue.list_position),
            ),
            model.ws.as_ref(),
            orders,
        );
    }
    project_page.issue_drag.clear();
}

pub fn change_status(status_id: IssueStatusId, model: &mut Model) {
    let project_page = match &mut model.page_content {
        PageContent::Project(project_page) => project_page,
        _ => return,
    };

    let issue_id = match project_page.issue_drag.dragged_id.as_ref().cloned() {
        Some(issue_id) => issue_id,
        _ => return error!("Nothing is dragged"),
    };

    let mut old: Vec<Issue> = vec![];
    let mut pos = 0;
    let mut found: Option<Issue> = None;
    std::mem::swap(&mut old, &mut model.issues);
    old.sort_by(|a, b| a.list_position.cmp(&b.list_position));

    for mut issue in old.into_iter() {
        if issue.issue_status_id == status_id {
            if issue.list_position != pos {
                issue.list_position = pos;
                project_page.issue_drag.mark_dirty(issue.id);
            }
            pos += 1;
        }
        if issue.id != issue_id {
            model.issues.push(issue);
        } else {
            found = Some(issue);
        }
    }

    let mut issue = match found {
        Some(i) => i,
        _ => {
            return;
        }
    };

    if issue.issue_status_id == status_id {
        model.issues.push(issue);
    } else {
        issue.issue_status_id = status_id;
        issue.list_position = pos + 1;
        model.issues.push(issue);
        project_page.issue_drag.mark_dirty(issue_id);
    }
}
