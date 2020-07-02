mod castle;
mod disaster;
mod game;
mod server;

use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct JoinGameEvent {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    lambda!(my_handler);
    Ok(())
}

fn my_handler(e: JoinGameEvent, c: Context) -> Result<CustomOutput, HandlerError> {
    Ok(CustomOutput {
        message: format!("Hello, {}!", "from the other side!"),
    })
}
