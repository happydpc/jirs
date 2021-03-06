use actix::{Handler, Message};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use jirs_data::{Token, User};

use crate::db::{DbExecutor, DbPool, SyncQuery};
use crate::errors::ServiceErrors;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeUser {
    pub access_token: uuid::Uuid,
}

impl Message for AuthorizeUser {
    type Result = Result<User, ServiceErrors>;
}

impl Handler<AuthorizeUser> for DbExecutor {
    type Result = Result<User, ServiceErrors>;

    fn handle(&mut self, msg: AuthorizeUser, _: &mut Self::Context) -> Self::Result {
        use crate::schema::tokens::dsl::{access_token, tokens};
        use crate::schema::users::dsl::{id, users};

        let conn = &self
            .pool
            .get()
            .map_err(|_| ServiceErrors::DatabaseConnectionLost)?;

        let token = tokens
            .filter(access_token.eq(msg.access_token))
            .first::<Token>(conn)
            .map_err(|_e| {
                ServiceErrors::RecordNotFound(format!("token for {}", msg.access_token))
            })?;

        let user_query = users.filter(id.eq(token.user_id));
        debug!("{}", diesel::debug_query::<Pg, _>(&user_query));
        user_query
            .first::<User>(conn)
            .map_err(|_e| ServiceErrors::RecordNotFound(format!("user {}", token.user_id)))
    }
}

impl SyncQuery for AuthorizeUser {
    type Result = std::result::Result<User, crate::errors::ServiceErrors>;

    fn handle(&self, pool: &DbPool) -> Self::Result {
        use crate::schema::tokens::dsl::{access_token, tokens};
        use crate::schema::users::dsl::{id, users};

        let conn = pool
            .get()
            .map_err(|_| crate::errors::ServiceErrors::DatabaseConnectionLost)?;
        let token = tokens
            .filter(access_token.eq(self.access_token))
            .first::<Token>(&conn)
            .map_err(|_| crate::errors::ServiceErrors::Unauthorized)?;

        let user_query = users.filter(id.eq(token.user_id));
        debug!("{}", diesel::debug_query::<Pg, _>(&user_query));
        user_query
            .first::<User>(&conn)
            .map_err(|_| crate::errors::ServiceErrors::Unauthorized)
    }
}
