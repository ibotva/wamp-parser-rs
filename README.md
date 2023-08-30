# Wamp Protocol Helpers
This ***is not*** a WAMP Library. This ***is*** a library for parsing WAMP messages. This repo requires the `json` crate to work. The docs this repo was based on can be found at [WAMP Protocol](https://github.com/Raynes/WAMP/blob/master/spec/basic.md).

## Cargo.toml
```toml
wamp_helpers = { git = "https://github.com/ibotva/wamp-parser-rs.git", branch = "main" }
```

## Usage
```rs
// Create a Hello message to Send to the WAMP server
let hello_message = messages::Hello {
    realm: "some_realm".to_string(),
    details: json::object!{
        authmethods: None,
        roles: {
            publisher: {},
            subscriber: {},
            caller: {},
            calee: {}
        }
    }
};

// Convert the Hello message to a JSON representation. (This consumes ownership of hello_message)
let message = hello_message.to_json().unwrap();

// Convert the JSON message to a String
message.to_string();

// Get the message permissions for a given role
// This returns an object with two bools, indicating whether the given Role is allowed to either receive, or send a message.
messages::Hello::get_message_direction(Roles::Broker);

// Example parsing a raw WAMP message string
// It can error for a variety of reasons, either due to json parsing, or WAMP violations
//the returned value will be a enum member of Events representing the type of frame.
let parsed = messages::Events::parse_message(message.to_string()).unwrap();

// For example if you were to pass a Hello message in the above example, you could for example use this code to handle Hello frames
if let messages::Events::Hello(hello) = parsed {
    println!("{}", hello.realm)
}

// However in most cases you would prefer to handle the error so heres how
match messages::Events::parse_message(message.to_string()) {
    Ok(event) => {
        match event {
            Hello(hello) => {
                // do something with Hello struct
            }

            Abort(abort) => {
                // happens sometimes after wamp connects to abandon the connection.
            }

            MessageError(error) => {
                // this is a wamp error from the server, not a library error because it follows WAMP spec, in this example because our string is constructed using the library it is not possible that the hello message can match to this enum, but it is here as an example.
            }

            _ => {
                // default logic if the frame does not match these.
            }
        }
    },
    Err(error) => {
        // Handle library error, either wamp protocol related, or JSON related, an example would be to check if the error is from the json library or if it is from wamp protocol.
        match error {
            ExtensionMessage => {} // this happens when it follows WAMP protocol but does not exist on our implementation

            InvalidId => {} // Happens when the ID fails to parse as a u8

            _ => {}
        }
    }
}
```
