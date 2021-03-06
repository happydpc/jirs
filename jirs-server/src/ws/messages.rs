use futures::executor::block_on;

use jirs_data::{MessageId, WsMsg};

use crate::db::messages;
use crate::ws::{WebSocketActor, WsHandler, WsResult};

pub struct LoadMessages;

impl WsHandler<LoadMessages> for WebSocketActor {
    fn handle_msg(&mut self, _msg: LoadMessages, _ctx: &mut Self::Context) -> WsResult {
        let user_id = self.require_user()?.id;
        match block_on(self.db.send(messages::LoadMessages { user_id })) {
            Ok(Ok(v)) => Ok(Some(WsMsg::MessagesResponse(v))),
            Ok(Err(e)) => {
                error!("{:?}", e);
                Ok(None)
            }
            Err(e) => {
                error!("{}", e);
                Ok(None)
            }
        }
    }
}

pub struct MarkMessageSeen {
    pub id: MessageId,
}

impl WsHandler<MarkMessageSeen> for WebSocketActor {
    fn handle_msg(&mut self, msg: MarkMessageSeen, _ctx: &mut Self::Context) -> WsResult {
        let user_id = self.require_user()?.id;
        match block_on(self.db.send(messages::MarkMessageSeen {
            message_id: msg.id,
            user_id,
        })) {
            Ok(Ok(id)) => Ok(Some(WsMsg::MessageMarkedSeen(id))),
            Ok(Err(e)) => {
                error!("{:?}", e);
                Ok(None)
            }
            Err(e) => {
                error!("{}", e);
                Ok(None)
            }
        }
    }
}
