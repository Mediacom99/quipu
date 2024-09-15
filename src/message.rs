//! Handles ChatMessage struct, its serialization and deserialization

use crate::prelude::*;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;

/// Handles a basic chat message, only text.
/// Needs serde traits to that bincode knows how to treat them.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    ///author of the message
    pub author: String,

    ///content of the message
    pub content: String,

    /// timestamp of the message creation
    pub datetime: DateTime<Local>,

    /// identifier of the chatroom
    /// to which the message belongs to
    pub room_id: String,
}

///Custom errors for messages
#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Failed to serialize message: {0}")]
    SerializationError(String),
    #[error("Failed to deserialize message: {0}")]
    DeserializationError(String),
}

impl ChatMessage {
    ///Create a new ChatMessage given the author, content and chatroom id
    ///the timestap refers to the local time of the machine that created
    ///the message.
    pub fn new(author: String, content: String, room_id: String) -> ChatMessage {
        trace!("{} created a new message", author);
        Self {
            author,
            content,
            datetime: Local::now(),
            room_id,
        }
    }

    #[instrument]
    ///Serialize ChatMessage struct into vector of u8 using bincode.
    pub async fn serialize(&self) -> Result<Vec<u8>, MessageError> {
        bincode::serialize(self).map_err(|err| {
            error!("could not serialize the message: {:#?}", self);
            MessageError::SerializationError(err.to_string())
        })
    }
    
    #[instrument]
    ///Serialize ChatMessage struct into vector of u8 using bincode.
    pub async fn deserialize(&self, data: &[u8]) -> Result<Self, MessageError> {
        bincode::deserialize(data).map_err(|err| {
            error!("could not deserialize the message: {:#?}", self);
            MessageError::DeserializationError(err.to_string())
        })
    }
}

#[tokio::test]
#[instrument]
///Test for ChatMessage serialization/deserialization using bincode
async fn test_chatmessage_serialization() {
    crate::log::tracing_subscriber_setup("trace").await;
    
    let newmessage = ChatMessage::new(
        "Edoardo".to_string(),
        "This is a message from Edoardo".to_string(),
        "6969".to_string(),
    );

    let newmessage2 = ChatMessage::new(
        "Pietro".to_string(),
        "dasdasd".to_string(),
        "1234".to_string(),
    );
    
    debug!("New message: {:#?}", newmessage);

    let data = newmessage.serialize().await.unwrap();
    debug!("Serialized message: {:?}", data);

    let oldmessage = newmessage.deserialize(data.as_slice()).await.unwrap();

    debug!("Deserialized message: {:#?}", oldmessage);

    assert_ne!(newmessage, newmessage2);
    assert_eq!(newmessage, oldmessage);

}
