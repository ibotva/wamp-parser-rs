# Wamp Protocol Helpers
This ***is not*** a WAMP Library. This ***is*** a library for parsing WAMP messages. This repo requires the `json` crate to work. The docs this repo was based on can be found at [WAMP Protocol](https://github.com/Raynes/WAMP/blob/master/spec/basic.md).

## Cargo.toml
```toml
wamp-helpers = { git = "https://github.com/ibotva/wamp-parser-rs.git", branch = "main" }
```

## Usage
```rs
use wamp_helpers::messages::{
    Events, Roles,

    Hello, WampMessageTrait,
};

use wamp_helpers::error::Error;

use json::{
    Null,
    object
};

fn main() {
    // Create a Hello message to Send to the WAMP server
    let hello_message = Hello{
        realm: "some.fun.realm".to_string(),
        details: object!{
            authmethods: Null,
            roles: {
                publisher: {},
                subscriber: {},
                caller: {},
                callee: {}
            }
        }
    };

    // Convert the Hello message to a JSON representation. (This consumes ownership of hello_message)
    let message = hello_message.to_json().unwrap();

    // Convert the JSON message to a String
    let json_hello = &message.to_string();

    // Example parsing a raw WAMP message string
    // It can error for a variety of reasons, either due to json parsing, or WAMP violations
    //the returned value will be a enum member of Events representing the type of frame.
    let parsed_json = Events::parse_message(json_hello);

    // For example if you were to pass a Hello message in the above example, you could for example use this code to handle Hello frames
    match parsed_json {
        Ok(event) => {
            match event {
                // do something with Hello struct
                Events::Hello(hello) => {
                    println!("{}", hello.realm)
                }

                // this is a wamp error from the server, not a library error because it follows WAMP spec, in this example because our string is constructed using the library it is not possible that the hello message can match to this enum, but it is here as an example.
                Events::ErrorMessage(err) => {
                    println!("{}", err.error)
                }

                _ => {} // default logic if the frame does not match these.
            }
        },
        // Handle library error, either wamp protocol related, or JSON related, an example would be to check if the error is from the json library or if it is from wamp protocol.
        Err(err) => {
            match err {
                Error::ExtensionMessage => {}, // this happens when it follows WAMP protocol but does not exist on our implementation
                Error::InvalidId => {},  // Happens when the ID fails to parse as a u8
                _ => {}
            }
        }
    }

    // Get the message permissions for a given role
    // This returns an object with two bools, indicating whether the given Role is allowed to either receive, or send a message.
    let _role_permissions = Hello::get_message_direction(Roles::Subscriber);
    
    
}
```
