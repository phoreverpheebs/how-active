use std::{io, env, process};

use reqwest::StatusCode;

mod app;
mod discord;

fn main() -> io::Result<()> {
    let token = match env::args().skip(1).next() {
        Some(val) => val,
        None => match env::var("DISCORD_TOKEN") {
            Ok(val) => val,
            Err(_) => usage(1),
        },
    };

    if token.is_empty() {
        usage(1);
    }

    app::deploy(
        &mut match app::App::new(token) {
            Ok(a) => a,
            Err(e) => {
                match e.status() {
                    Some(StatusCode::UNAUTHORIZED) => eprintln!("Invalid auth token"),
                    _ => eprintln!("Error: {}", e),
                }
                process::exit(1);
            },
        }
    )
}

fn usage(exit_code: i32) -> ! {
    eprintln!("
    Usage: how-active [token]

    token => Your Discord authorization token (can be grabbed from request headers of Discord API requests)
        Either add environment variable `DISCORD_TOKEN` with your token, or provide it as an argument.
");
    process::exit(exit_code);
}
