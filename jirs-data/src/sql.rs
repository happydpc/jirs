use std::io::Write;

use diesel::{deserialize::*, pg::*, serialize::*, *};

use crate::{IssuePriority, IssueStatus, IssueType};

#[derive(SqlType)]
#[postgres(type_name = "IssuePriorityType")]
pub struct IssuePriorityType;

impl ToSql<IssuePriorityType, Pg> for IssuePriority {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            IssuePriority::Highest => out.write_all(b"highest")?,
            IssuePriority::High => out.write_all(b"high")?,
            IssuePriority::Medium => out.write_all(b"medium")?,
            IssuePriority::Low => out.write_all(b"low")?,
            IssuePriority::Lowest => out.write_all(b"lowest")?,
        }
        Ok(IsNull::No)
    }
}

fn issue_priority_from_sql(bytes: Option<&[u8]>) -> deserialize::Result<IssuePriority> {
    match not_none!(bytes) {
        b"5" | b"highest" => Ok(IssuePriority::Highest),
        b"4" | b"high" => Ok(IssuePriority::High),
        b"3" | b"medium" => Ok(IssuePriority::Medium),
        b"2" | b"low" => Ok(IssuePriority::Low),
        b"1" | b"lowest" => Ok(IssuePriority::Lowest),
        _ => Ok(IssuePriority::Lowest),
    }
}

impl FromSql<IssuePriorityType, Pg> for IssuePriority {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        issue_priority_from_sql(bytes)
    }
}

impl FromSql<sql_types::Text, Pg> for IssuePriority {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        issue_priority_from_sql(bytes)
    }
}

#[derive(SqlType)]
#[postgres(type_name = "IssueTypeType")]
pub struct IssueTypeType;

fn issue_type_from_sql(bytes: Option<&[u8]>) -> deserialize::Result<IssueType> {
    match not_none!(bytes) {
        b"task" => Ok(IssueType::Task),
        b"bug" => Ok(IssueType::Bug),
        b"story" => Ok(IssueType::Story),
        _ => Ok(IssueType::Task),
    }
}

impl FromSql<IssueTypeType, Pg> for IssueType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        issue_type_from_sql(bytes)
    }
}

impl FromSql<sql_types::Text, Pg> for IssueType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        issue_type_from_sql(bytes)
    }
}

impl ToSql<IssueTypeType, Pg> for IssueType {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            IssueType::Task => out.write_all(b"task")?,
            IssueType::Story => out.write_all(b"story")?,
            IssueType::Bug => out.write_all(b"bug")?,
        }
        Ok(IsNull::No)
    }
}

#[derive(SqlType)]
#[postgres(type_name = "IssueStatusType")]
pub struct IssueStatusType;

impl diesel::query_builder::QueryId for IssueStatusType {
    type QueryId = IssueStatus;
}

fn issue_status_from_sql(bytes: Option<&[u8]>) -> deserialize::Result<IssueStatus> {
    match not_none!(bytes) {
        b"backlog" => Ok(IssueStatus::Backlog),
        b"selected" => Ok(IssueStatus::Selected),
        b"in_progress" | b"inprogress" => Ok(IssueStatus::InProgress),
        b"done" => Ok(IssueStatus::Done),
        _ => Ok(IssueStatus::Backlog),
    }
}

impl FromSql<IssueStatusType, Pg> for IssueStatus {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        issue_status_from_sql(bytes)
    }
}

impl FromSql<sql_types::Text, Pg> for IssueStatus {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        issue_status_from_sql(bytes)
    }
}

impl ToSql<IssueStatusType, Pg> for IssueStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            IssueStatus::Backlog => out.write_all(b"backlog")?,
            IssueStatus::Selected => out.write_all(b"selected")?,
            IssueStatus::InProgress => out.write_all(b"in_progress")?,
            IssueStatus::Done => out.write_all(b"done")?,
        }
        Ok(IsNull::No)
    }
}