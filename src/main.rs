pub mod llm;
use clap::{ App, Arg };
use dotenv::dotenv;
use llm::*;
use copypasta::{ ClipboardContext, ClipboardProvider };
use std::io::{ self, Read };
use atty::Stream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let mut ctx = ClipboardContext::new().expect("Failed to initialize clipboard");

    let matches = App::new("terminal-reader")
        .version("1.0")
        .arg(
            Arg::with_name("pat")
                .long("pattern")
                .help("Specifies the pattern to watch")
                .takes_value(true)
        )
        .get_matches();

    let pat = matches.value_of("pat").unwrap_or(".*");
    println!("Value for pattern: {}", pat);

    let mut input = String::new();
    if atty::is(atty::Stream::Stdin) {
        println!("No input piped in.");
    } else {
        io::stdin().read_to_string(&mut input)?;
    }

    let system_prompt = format!(
        "You're a coding bot, you're tasked to read terminal outs and identify potential issues."
    );
    let user_input = format!(
        "These are the output of the program executed: `{input}`, here is the area to pay special attention to : `{pat}`"
    );

    let res = chat_inner_async(&system_prompt, &input, 200, "gpt-3.5-turbo").await?;

    println!("step 2 Raw: {:?}", res);

    Ok(())
}
