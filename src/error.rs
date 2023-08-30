use json::JsonValue;

#[derive(Debug)]
pub enum Error {
    UrlParseError(websocket::url::ParseError),
    WebSocketError(websocket::WebSocketError),
    DefaultImplementationError(&'static str),
    JsonError(json::Error),
    InvalidId,
    ExtensionMessage,
    NonMatchingMessageId { offense: u8 },
    InvalidJsonU8 {offense: JsonValue},
    InvalidJsonDict {offense: JsonValue},
    InvalidJsonArray {offense: JsonValue},
    InvalidJsonU64 {offense: JsonValue},
    InvalidJsonStr {offense: JsonValue}
}

