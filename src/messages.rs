use std::str::FromStr;
use crate::error::Error;
use json::JsonValue;

pub type WampId = u64;
pub type Uri = String;

#[derive(Debug)]
pub enum Roles {
    Callee,
    Caller,
    Publisher,
    Subscriber,
    Dealer, // Dealer performs routing between Callee and Caller, A broker performs routing of events from publishers to subscribers.
    Broker
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct MessageDirection {
    pub receives: &'static bool,
    pub sends: &'static bool
}

pub type Args = JsonValue;
pub type Kwargs = JsonValue;
pub type Details = JsonValue;
pub type Options = JsonValue;

fn validate_u64_argument(value: JsonValue) -> Result<u64, Error> {
    if let Some(value) = value.as_u64() {
        Ok(value)
    } else {
        Err(Error::InvalidJsonU64 { offense: value })
    }
}

fn validate_dict_argument(value: JsonValue) -> Result<JsonValue, Error> {
    if value.is_object() {
        Ok(value)
    } else {
        Err(Error::InvalidJsonDict { offense: value })
    }
}

fn validate_array_argument(value: JsonValue) -> Result<JsonValue, Error> {
    if value.is_array() {
        Ok(value)
    } else {
        Err(Error::InvalidJsonArray { offense: value })
    }
}

fn validate_u8_argument(value: JsonValue) -> Result<u8, Error> {
    if let Some(id) = value.as_u8() {
        Ok(id)
    } else {
        Err(Error::InvalidJsonU8 { offense: value })
    }
}

fn validate_str_argument(value: JsonValue) -> Result<String, Error> {
    if let Some(value) = value.as_str() {
        Ok(value.to_string())
    } else {
        Err(Error::InvalidJsonStr { offense: value })
    }
}

fn validate_args(value: JsonValue) -> Result<Option<JsonValue>, Error> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(validate_array_argument(value)?))
    }
}

fn validate_kwargs(value: JsonValue) -> Result<Option<JsonValue>, Error> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(validate_dict_argument(value)?))
    }
}

pub trait WampMessageTrait {
    const ID: u8;

    fn to_json(self) -> Result<JsonValue, Error>;

    fn get_message_direction(role: Roles) -> &'static MessageDirection where Self:Sized;


    fn validate_id(value: JsonValue) -> Result<u8, Error> {
        if let Some(id) = value.as_u8() {
            if Self::ID == id {
                Ok(id)
            } else {
                Err(Error::NonMatchingMessageId { offense: id })
            }
        } else {
            Err(Error::InvalidJsonU8 { offense: value })
        }
    }

    fn parse_raw_json(data: String) -> Result<JsonValue, Error> {
        match json::parse(&data) {
            Ok(new_data) => Ok(new_data),
            Err(err) => Err(Error::JsonError(err))
        }
    }

}

#[derive(Debug, Clone)]
pub struct Hello {
    pub realm: Uri,
    pub details: Details
}

impl Hello {
    /// Create a help message with default details object containing roles and auth methods.
    /// # Examples
    /// ```
    /// use wamp_v1::messages::{Hello, Roles};
    /// let hello = Hello::default(
    ///     "some.realm.uri".to_string(), 
    ///     vec![Roles::Callee, Roles::Caller, Roles::Publisher, Roles::Subscriber],
    ///     Some(vec!["ticket".to_string()]) // Should be `None` for non advanced configurations
    /// );
    /// ```
    pub fn default(realm: String, roles: Vec<Roles>, authmethods: Option<Vec<String>>) -> Self {
        let mut details = json::object!{
            roles: {

            }
        };
        
        for role in roles {
            match role {
                Roles::Callee => details["roles"]["callee"] = json::object!{},
                Roles::Caller => details["roles"]["caller"] = json::object!{},
                Roles::Publisher => details["roles"]["publisher"] = json::object!{},
                Roles::Subscriber => details["roles"]["subscriber"] = json::object!{},
                Roles::Dealer => details["roles"]["dealer"] = json::object!{},
                Roles::Broker => details["roles"]["broker"] = json::object!{},
            }
        };

        if let Some(authmethods) = authmethods {
            details["authmethods"] = json::array![];
            for method in authmethods {
                let _ = details["authmethods"].push(method);
            }
        };

        Hello { realm, details }
        
    }

}


impl WampMessageTrait for Hello {
    const ID: u8 = 1;
    ///```
    /// use wamp_v1::messages::{Hello, ToJson};
    /// use json::object;
    /// // To create a new Hello Message 
    /// let hello = Hello {
    ///     realm: "some.uri.path".to_string(),
    ///     details: object!{
    ///         authmethods: ["ticket"], // For advanced wamp configurations
    ///         roles: { // Roles are required by Wamp
    ///                 "publisher": {},
    ///                 "subscriber": {},
    ///                 "caller": {},
    ///                 "callee": {}
    ///         }
    ///     }
    /// };
    /// 
    /// // This converts it to a websocket readable message.
    /// let message = hello.to_json().to_string();
    /// 
    /// print!("{}", message.to_string());
    /// ```
    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.realm,
            self.details])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &true },
            Roles::Caller => &MessageDirection { receives: &false, sends: &true },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &true },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &true },
            Roles::Dealer => &MessageDirection { receives: &true, sends: &false },
            Roles::Broker => &MessageDirection { receives: &true, sends: &false },
        }
    }
}

impl FromStr for Hello {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let realm = validate_str_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        Ok(Hello { realm, details })
    }
}


#[derive(Debug, Clone)]
pub struct Welcome {
    pub session: u64,
    pub details: Details
}

impl WampMessageTrait for Welcome {
    const ID: u8 = 2;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.session,
            self.details
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &true, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &true, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &true },
        }
    }
}



impl FromStr for Welcome {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let session = validate_u64_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        Ok(Welcome {
            session,
            details
        })
    }
}

#[derive(Debug, Clone)]
pub struct Abort {
    pub details: Details,
    pub reason: Uri
}

impl WampMessageTrait for Abort {
    const ID: u8 = 3;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.details,
            self.reason
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &true, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &true, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &true }
        }
    }
}

impl FromStr for Abort {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        let reason = validate_str_argument(data.array_remove(0))?;
        Ok(Abort { details, reason })
    }   
}

#[derive(Debug, Clone)]
pub struct Goodbye {
    pub details: Details,
    pub reason: Uri
}

impl WampMessageTrait for Goodbye {
    const ID: u8 = 6;
    
    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.details,
            self.reason
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &true },
            Roles::Caller => &MessageDirection { receives: &true, sends: &true },
            Roles::Publisher => &MessageDirection { receives: &true, sends: &true },
            Roles::Subscriber => &MessageDirection { receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &true, sends: &true },
            Roles::Broker => &MessageDirection { receives: &true, sends: &true },
        }
    }
}

impl FromStr for Goodbye {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        let reason = validate_str_argument(data.array_remove(0))?;
        Ok(Goodbye{
            details,
            reason
        })
        
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMessage {
    pub request_type: u8,
    pub request: WampId,
    pub details: Details,
    pub error: Uri,
    pub args: Option<Args>,
    pub kwargs: Option<Kwargs>
}

impl WampMessageTrait for ErrorMessage {
    const ID: u8 = 8;

    fn to_json(self) -> Result<JsonValue, Error> {
        let mut data = json::array![
            Self::ID,
            self.request_type,
            self.request,
            self.details,
            self.error
        ];

        let is_array = if let Some(args) = self.args {
            let n = args.is_array();
            if n {
                data.push(args)
                    .map_err(|err| Error::JsonError(err))?;
            }
            n
        } else {
            false
        };
        if let Some(kwargs) = self.kwargs {
            if kwargs.is_object() {
                if !is_array {
                    data.push(json::array![])
                        .map_err(|err| Error::JsonError(err))?;
                }
                
                data.push(kwargs)
                    .map_err(|err| Error::JsonError(err))?;
                
            };
        }
        Ok(data)
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &true },
            Roles::Caller => &MessageDirection { receives: &true, sends: &false },
            Roles::Publisher =>  &MessageDirection { receives: &true, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &true, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &true },
        }
    }
}

impl FromStr for ErrorMessage {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = validate_u8_argument(data.array_remove(0))?;
        let request_type = validate_u8_argument(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        let error = validate_str_argument(data.array_remove(0))?;
        let args = validate_args(data.array_remove(0))?;
        let kwargs = validate_kwargs(data.array_remove(0))?;
        Ok(Self { request_type, request, details, error, args, kwargs })
    }
}

#[derive(Debug, Clone)]
pub struct Publish {
    pub request: WampId,
    pub options: Options,
    pub topic: Uri,
    pub args: Option<Args>,
    pub kwargs: Option<Kwargs>
}

impl WampMessageTrait for Publish {
    const ID: u8 = 16;

    fn to_json(self) -> Result<JsonValue, Error> {
        let mut data = json::array![
            Self::ID,
            self.request,
            self.options,
            self.topic
        ];
        let is_array = if let Some(args) = self.args {
            let n = args.is_array();
            if n {
                data.push(args)
                    .map_err(|err| Error::JsonError(err))?;
            }
            n
        } else {
            false
        };
        if let Some(kwargs) = self.kwargs {
            if kwargs.is_object() {
                if !is_array {
                    data.push(json::array![])
                        .map_err(|err| Error::JsonError(err))?;
                }
                
                data.push(kwargs)
                    .map_err(|err| Error::JsonError(err))?;
                
            };
        }
        Ok(data)
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &true },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &true },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &false },
            Roles::Broker => &MessageDirection { receives: &true, sends: &false },
        }
    }
}

impl FromStr for Publish {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let id = validate_u8_argument(data.array_remove(0))?;
        if id != Self::ID {
            Err(Error::NonMatchingMessageId { offense: id })
        } else {
            let request = validate_u64_argument(data.array_remove(0))?;
            let options = validate_dict_argument(data.array_remove(0))?;
            let topic = validate_str_argument(data.array_remove(0))?;
            let args = validate_args(data.array_remove(0))?;
            let kwargs = validate_kwargs(data.array_remove(0))?;
            Ok(Publish {
                request,
                options,
                topic,
                args,
                kwargs
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Published {
    pub request: WampId,
    pub publication: WampId
}

impl WampMessageTrait for Published {
    const ID: u8 = 17;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request,
            self.publication
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &true, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &false },
            Roles::Broker => &MessageDirection { receives: &false, sends: &true },
        }
    }
}

impl FromStr for Published {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = validate_u8_argument(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let publication = validate_u64_argument(data.array_remove(0))?;
        Ok(Published {
            request,
            publication,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Subscribe {
    pub request: WampId,    
    pub options: Options,
    pub topic: Uri
}

impl WampMessageTrait for Subscribe {
    const ID: u8 = 32;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.options,
            self.topic
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &false },
            Roles::Caller =>  &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &true },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &false },
            Roles::Broker => &MessageDirection { receives: &true, sends: &false },
        }
    }
}

impl FromStr for Subscribe {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let options = validate_dict_argument(data.array_remove(0))?;
        let topic = validate_str_argument(data.array_remove(0))?;
        Ok(Subscribe {
            request,
            options,
            topic
        })

    }
}

#[derive(Debug, Clone)]
pub struct Subscribed {
    pub request: WampId,
    pub subscription: WampId
}

impl WampMessageTrait for Subscribed {
    const ID: u8 = 33;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request,
            self.subscription
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &false },
            Roles::Broker => &MessageDirection { receives: &false, sends: &true },
        }
    }
}


impl FromStr for Subscribed {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let subscription = validate_u64_argument(data.array_remove(0))?;
        Ok(Subscribed { request, subscription })
    }
}

#[derive(Debug, Clone)]
pub struct Unsubscribe {
    pub request: WampId,
    pub subscription: WampId
}

impl WampMessageTrait for Unsubscribe {
    const ID: u8 = 34;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request,
            self.subscription
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &true },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &false },
            Roles::Broker => &MessageDirection { receives: &true, sends: &false },
        }
    }
}

impl FromStr for Unsubscribe {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let subscription = validate_u64_argument(data.array_remove(0))?;
        Ok(Unsubscribe { request, subscription })
    }
}

pub struct Unsubscribed {
    request: WampId
}

impl WampMessageTrait for Unsubscribed {
    const ID: u8 = 35;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection where Self:Sized {
        match role {
            Roles::Callee => &MessageDirection{ receives: &false, sends: &false },
            Roles::Caller => &MessageDirection{ receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection{ receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection{ receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection{ receives: &false, sends: &false },
            Roles::Broker => &MessageDirection{ receives: &false, sends: &true },
        }
    }
}

impl FromStr for Unsubscribed {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let request = validate_u64_argument(data.array_remove(0))?;
        Ok(Unsubscribed { request })
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub subscription: WampId,
    pub publication: WampId,
    pub details: Details,
    pub args: Option<Args>,
    pub kwargs: Option<Kwargs>
}

impl WampMessageTrait for Event {
    const ID: u8 = 36;

    fn to_json(self) -> Result<JsonValue, Error> {
        let mut data = json::array![
            Self::ID,
            self.subscription,
            self.publication,
            self.details
        ];
        let is_array = if let Some(args) = self.args {
            let n = args.is_array();
            if n {
                data.push(args)
                    .map_err(|err| Error::JsonError(err))?;
            }
            n
        } else {
            false
        };
        if let Some(kwargs) = self.kwargs {
            if kwargs.is_object() {
                if !is_array {
                    data.push(json::array![])
                        .map_err(|err| Error::JsonError(err))?;
                }
                
                data.push(kwargs)
                    .map_err(|err| Error::JsonError(err))?;
                
            };
        }
        Ok(data)
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &false },
            Roles::Broker => &MessageDirection { receives: &false, sends: &true },
        }
    }
}

impl FromStr for Event {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let subscription = validate_u64_argument(data.array_remove(0))?;
        let publication = validate_u64_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        let args = validate_args(data.array_remove(0))?;
        let kwargs = validate_kwargs(data.array_remove(0))?;
        Ok(Event {
            subscription,
            publication,
            details,
            args,
            kwargs
        })
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub request: WampId,
    pub options: Options,
    pub procedure: Uri,
    pub args: Option<Args>,
    pub kwargs: Option<Kwargs>
}

impl WampMessageTrait for Call {
    const ID: u8 = 48;

    fn to_json(self) -> Result<JsonValue, Error> {
        let mut data = json::array![
            Self::ID,
            self.request,
            self.options,
            self.procedure
        ];

        let is_array = if let Some(args) = self.args {
            let n = args.is_array();
            if n {
                data.push(args)
                    .map_err(|err| Error::JsonError(err))?;
            }
            n
        } else {
            false
        };
        if let Some(kwargs) = self.kwargs {
            if kwargs.is_object() {
                if !is_array {
                    data.push(json::array![])
                        .map_err(|err| Error::JsonError(err))?;
                }
                
                data.push(kwargs)
                    .map_err(|err| Error::JsonError(err))?;
                
            };
        }
        Ok(data)
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &true },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}


impl FromStr for Call {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let options = validate_dict_argument(data.array_remove(0))?;
        let procedure = validate_str_argument(data.array_remove(0))?;
        let args = validate_args(data.array_remove(0))?;
        let kwargs = validate_kwargs(data.array_remove(0))?;
        Ok(Call {
            request,
            options,
            procedure,
            args,
            kwargs
        })
    }
}

#[derive(Debug, Clone)]
pub struct MessageResult {
    pub request: WampId,
    pub details: Details,
    pub args: Option<Args>,
    pub kwargs: Option<Kwargs>
}

impl WampMessageTrait for MessageResult {
    const ID: u8 = 50;

    fn to_json(self) -> Result<JsonValue, Error> {
        let mut data = json::array![
            Self::ID,
            self.request,
            self.details
        ];

        let is_array = if let Some(args) = self.args {
            let n = args.is_array();
            if n {
                data.push(args)
                    .map_err(|err| Error::JsonError(err))?;
            }
            n
        } else {
            false
        };
        if let Some(kwargs) = self.kwargs {
            if kwargs.is_object() {
                if !is_array {
                    data.push(json::array![])
                        .map_err(|err| Error::JsonError(err))?;
                }
                
                data.push(kwargs)
                    .map_err(|err| Error::JsonError(err))?;
                
            };
        }
        Ok(data)
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &false },
            Roles::Caller =>  &MessageDirection { receives: &true, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &false },
            Roles::Broker =>  &MessageDirection { receives: &false, sends: &true },
        }
    }
}


impl FromStr for MessageResult {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        let args = validate_args(data.array_remove(0))?;
        let kwargs = validate_kwargs(data.array_remove(0))?;
        Ok(MessageResult { request, details, args, kwargs })
    }
}

#[derive(Debug, Clone)]
pub struct Register {
    pub request: WampId,
    pub options: Options,
    pub procedure: Uri
}

impl WampMessageTrait for Register {
    const ID: u8 = 64;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            self.request, 
            self.options, 
            self.procedure
        ])
    }
    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &true },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &true, sends: &false },
            Roles::Broker =>  &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Register {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let options = validate_dict_argument(data.array_remove(0))?;
        let procedure = validate_str_argument(data.array_remove(0))?;
        Ok(Register {
            request,
            options,
            procedure
        })
    }
}

#[derive(Debug, Clone)]
pub struct Registered {
    pub request: WampId,
    pub registration: WampId
}

impl WampMessageTrait for Registered {
    const ID: u8 = 65;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request,
            self.registration
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Registered {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let registration = validate_u64_argument(data.array_remove(0))?;
        Ok(Registered { request, registration })
    }
}

#[derive(Debug, Clone)]
pub struct Unregister {
    pub request: WampId,
    pub registration: WampId
}

impl WampMessageTrait for Unregister {
    const ID: u8 = 66;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request,
            self.registration
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &true },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber =>  &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives:&true, sends: &false },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Unregister {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let registration = validate_u64_argument(data.array_remove(0))?;
        Ok(Unregister { request, registration })
    }
}

#[derive(Debug, Clone)]
pub struct Unregistered {
    pub request: WampId
}

impl WampMessageTrait for Unregistered {
    const ID: u8 = 67;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Unregistered {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        Ok(Unregistered{
            request
        })
    }
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub request: WampId,
    pub registration: WampId,
    pub details: Details,
    pub args: Option<Args>,
    pub kwargs: Option<Kwargs>
}

impl WampMessageTrait for Invocation {
    const ID: u8 = 68;

    fn to_json(self) -> Result<JsonValue, Error> {
        let mut data = json::array![
            Self::ID,
            self.request,
            self.registration,
            self.details
        ];

        let is_array = if let Some(args) = self.args {
            let n = args.is_array();
            if n {
                data.push(args)
                    .map_err(|err| Error::JsonError(err))?;
            }
            n
        } else {
            false
        };
        if let Some(kwargs) = self.kwargs {
            if kwargs.is_object() {
                if !is_array {
                    data.push(json::array![])
                        .map_err(|err| Error::JsonError(err))?;
                }
                
                data.push(kwargs)
                    .map_err(|err| Error::JsonError(err))?;
                
            };
        }
        Ok(data)
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Invocation {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let registration = validate_u64_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        let args = validate_args(data.array_remove(0))?;
        let kwargs = validate_kwargs(data.array_remove(0))?;
        Ok(Invocation {
            request,
            registration,
            details,
            args,
            kwargs
        })
    }
}

#[derive(Debug, Clone)]
pub struct Yield {
    pub request: WampId,
    pub options: Options,
    pub args: Option<Args>,
    pub kwargs: Option<Kwargs>
}

impl WampMessageTrait for Yield {
    const ID: u8 = 70;

    fn to_json(self) -> Result<JsonValue, Error> {

        let mut data = json::array![
            Self::ID,
            self.options
        ];

        let is_array = if let Some(args) = self.args {
            let n = args.is_array();
            if n {
                data.push(args)
                    .map_err(|err| Error::JsonError(err))?;
            }
            n
        } else {
            false
        };

        if let Some(kwargs) = self.kwargs {
            if kwargs.is_object() {
                if !is_array {
                    data.push(json::array![])
                        .map_err(|err| Error::JsonError(err))?;
                }
                data.push(kwargs)
                    .map_err(|err| Error::JsonError(err))?;
                
            };
        }
        Ok(data)
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &true },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives:&true, sends: &false },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Yield {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let options = validate_dict_argument(data.array_remove(0))?;
        let args = validate_args(data.array_remove(0))?;
        let kwargs = validate_kwargs(data.array_remove(0))?;
        Ok(Yield{
            request,
            options,
            args,
            kwargs
        })
    }
}

#[derive(Debug, Clone)]
pub struct Challenge {
    authmethod: String,
    details: Kwargs
}

impl WampMessageTrait for Challenge {
    const ID: u8 = 4;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.authmethod,
            self.details
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &true, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &true, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &true, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &true },
        }
    }
}

impl FromStr for Challenge {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let authmethod = validate_str_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        Ok(Challenge{
            authmethod,
            details
        })
    }
}

#[derive(Debug, Clone)]
pub struct Authenticate {
    signature: String,
    details: Kwargs
}

impl WampMessageTrait for Authenticate {
    const ID: u8 = 5;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.signature,
            self.details
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &true },
            Roles::Caller => &MessageDirection { receives: &false, sends: &true },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &true },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &true },
            Roles::Dealer => &MessageDirection { receives: &true, sends: &false },
            Roles::Broker => &MessageDirection { receives: &true, sends: &false },
        }
    }
}

impl FromStr for Authenticate {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let signature = validate_str_argument(data.array_remove(0))?;
        let details = validate_dict_argument(data.array_remove(0))?;
        Ok(Authenticate{
            signature,
            details
        })
    }
}

#[derive(Debug, Clone)]
pub struct Cancel {
    request: WampId,
    options: Options
}

impl WampMessageTrait for Cancel {
    const ID: u8 = 49;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request,
            self.options
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &false, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &true },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Cancel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let options = validate_dict_argument(data.array_remove(0))?;
        Ok(Cancel { request, options })
    }
}

#[derive(Debug, Clone)]
pub struct Interrupt {
    request: WampId,
    options: Options
}

impl WampMessageTrait for Interrupt {
    const ID: u8 = 69;

    fn to_json(self) -> Result<JsonValue, Error> {
        Ok(json::array![
            Self::ID,
            self.request,
            self.options
        ])
    }

    fn get_message_direction(role: Roles) -> &'static MessageDirection {
        match role {
            Roles::Callee => &MessageDirection { receives: &true, sends: &false },
            Roles::Caller => &MessageDirection { receives: &false, sends: &false },
            Roles::Publisher => &MessageDirection { receives: &false, sends: &false },
            Roles::Subscriber => &MessageDirection { receives: &false, sends: &false },
            Roles::Dealer => &MessageDirection { receives: &false, sends: &true },
            Roles::Broker => &MessageDirection { receives: &false, sends: &false },
        }
    }
}

impl FromStr for Interrupt {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Self::parse_raw_json(s.to_string())?;
        let _id = Self::validate_id(data.array_remove(0))?;
        let request = validate_u64_argument(data.array_remove(0))?;
        let options = validate_dict_argument(data.array_remove(0))?;
        Ok(Interrupt{
            request,
            options
        })
    }
}

#[derive(Debug)]
pub enum Events {
    Hello(Hello),
    Welcome(Welcome),
    Abort(Abort),
    Challenge(Challenge),
    Authenticate(Authenticate),
    Goodbye(Goodbye),
    ErrorMessage(ErrorMessage),
    Publish(Publish),
    Published(Published),
    Subscribe(Subscribe),
    Subscribed(Subscribed),
    Unsubscribe(Unsubscribe),
    Unsubscribed(Unsubscribed),
    Event(Event),
    Call(Call),
    Cancel(Cancel),
    MessageResult(MessageResult),
    Register(Register),
    Registered(Registered),
    Unregister(Unregister),
    Unregistered(Unregistered),
    Invocation(Invocation),
    Interrupt(Interrupt),
    Yield(Yield)
}

impl Events {
    pub fn parse_message(raw_message_string: &String) -> Result<Self, Error> {
        let mut data = json::parse(raw_message_string)
            .map_err(|err| Error::JsonError(err))?;

        let id = data.array_remove(0).as_u8();
        
        if let Some(id) = id {
            match id {
                Hello::ID => {

                    let realm = validate_str_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;

                    Ok(Self::Hello(
                        Hello { realm, details }
                    ))
                }
                
                Welcome::ID => {

                    let session = validate_u64_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;

                    Ok(Self::Welcome(
                        Welcome { session, details }
                    ))
                }

                Abort::ID => {

                    let details = validate_dict_argument(data.array_remove(0))?;
                    let reason = validate_str_argument(data.array_remove(0))?;
                    Ok(Self::Abort(
                        Abort { details, reason }
                    ))

                }

                Challenge::ID => {
                    let authmethod = validate_str_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;
                    
                    Ok(Self::Challenge(
                        Challenge { authmethod, details }
                    ))
                }

                Authenticate::ID => {
                    let signature = validate_str_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;
                    Ok(Self::Authenticate(
                        Authenticate { signature, details }
                    ))
                }

                Goodbye::ID => {
                    let details = validate_dict_argument(data.array_remove(0))?;
                    let reason = validate_str_argument(data.array_remove(0))?;
                    Ok(Self::Goodbye(
                        Goodbye { details, reason }
                    ))
                }

                ErrorMessage::ID => {
                    let request_type = validate_u8_argument(data.array_remove(0))?;
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;
                    let error = validate_str_argument(data.array_remove(0))?;
                    let args = validate_args(data.array_remove(0))?;
                    let kwargs = validate_kwargs(data.array_remove(0))?;
                    Ok(Self::ErrorMessage(
                        ErrorMessage { request_type, request, details, error, args, kwargs }
                    ))
                }

                Publish::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let options = validate_dict_argument(data.array_remove(0))?;
                    let topic = validate_str_argument(data.array_remove(0))?;
                    let args = validate_args(data.array_remove(0))?;
                    let kwargs = validate_kwargs(data.array_remove(0))?;
                    Ok(Self::Publish(
                        Publish { request, options, topic, args, kwargs }
                    ))
                }

                Published::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let publication = validate_u64_argument(data.array_remove(0))?;
                    Ok(Self::Published(
                        Published { request, publication }
                    ))
                }

                Subscribe::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let options = validate_dict_argument(data.array_remove(0))?;
                    let topic = validate_str_argument(data.array_remove(0))?;
                    Ok(Self::Subscribe(
                        Subscribe { request, options, topic }
                    ))
                }

                Subscribed::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let subscription = validate_u64_argument(data.array_remove(0))?;
                    Ok(Self::Subscribed(
                        Subscribed { request, subscription }
                    ))
                }

                Unsubscribe::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let subscription: u64 = validate_u64_argument(data.array_remove(0))?;
                    Ok(Self::Unsubscribe(
                        Unsubscribe { request, subscription }
                    ))
                }

                Unsubscribed::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    Ok(Self::Unsubscribed(
                        Unsubscribed { request }
                    ))
                }

                Event::ID => {
                    let subscription = validate_u64_argument(data.array_remove(0))?;
                    let publication = validate_u64_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;
                    let args = validate_args(data.array_remove(0))?;
                    let kwargs = validate_kwargs(data.array_remove(0))?;
                    Ok(Self::Event(
                        Event { subscription, publication, details, args, kwargs }
                    ))
                }

                Call::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let options = validate_dict_argument(data.array_remove(0))?;
                    let procedure = validate_str_argument(data.array_remove(0))?;
                    let args = validate_args(data.array_remove(0))?;
                    let kwargs = validate_kwargs(data.array_remove(0))?;
                    Ok(Self::Call(
                        Call { request, options, procedure, args, kwargs }
                    ))
                }

                Cancel::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let options = validate_dict_argument(data.array_remove(0))?;
                    Ok(Self::Cancel(
                        Cancel { request, options }
                    ))
                }

                MessageResult::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;
                    let args = validate_args(data.array_remove(0))?;
                    let kwargs = validate_kwargs(data.array_remove(0))?;
                    Ok(Self::MessageResult(
                        MessageResult { request, details, args, kwargs }
                    ))
                }

                Register::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let options = validate_dict_argument(data.array_remove(0))?;
                    let procedure = validate_str_argument(data.array_remove(0))?;
                    Ok(Self::Register(
                        Register { request, options, procedure }
                    ))
                }

                Registered::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let registration = validate_u64_argument(data.array_remove(0))?;
                    Ok(Self::Registered(
                        Registered { request, registration }
                    ))
                }

                Unregister::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let registration = validate_u64_argument(data.array_remove(0))?;
                    Ok(Self::Unregister(
                        Unregister { request, registration }
                    ))
                }

                Unregistered::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    Ok(Self::Unregistered(
                        Unregistered { request }
                    ))
                }

                Invocation::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let registration = validate_u64_argument(data.array_remove(0))?;
                    let details = validate_dict_argument(data.array_remove(0))?;
                    let args = validate_args(data.array_remove(0))?;
                    let kwargs = validate_kwargs(data.array_remove(0))?;
                    Ok(Self::Invocation(
                        Invocation { request, registration, details, args, kwargs }
                    ))
                }

                Interrupt::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let options = validate_dict_argument(data.array_remove(0))?;
                    Ok(Self::Interrupt(
                        Interrupt { request, options }
                    ))
                }

                Yield::ID => {
                    let request = validate_u64_argument(data.array_remove(0))?;
                    let options = validate_dict_argument(data.array_remove(0))?;
                    let args = validate_args(data.array_remove(0))?;
                    let kwargs = validate_kwargs(data.array_remove(0))?;
                    Ok(Self::Yield(
                        Yield { request, options, args, kwargs }
                    ))
                }

                _ => {
                    Err(Error::ExtensionMessage)
                }
            }
        } else {
            Err(Error::InvalidId)
        }
    }

    pub fn is_basic(&self) -> bool {
        match self {
            Self::Challenge(_challenge) => false,
            Self::Authenticate(_authenticate) => false,
            Self::Cancel(_cancel) => false,
            Self::Interrupt(_interrupt) => false,
            _ => true
        }
    }

    pub fn is_advanced(&self) -> bool {
        match self {
            Self::Challenge(_challenge) => true,
            Self::Authenticate(_authenticate) => true,
            Self::Cancel(_cancel) => true,
            Self::Interrupt(_interrupt) => true,
            _ => false
        }
    }
}
