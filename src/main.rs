pub mod llm_low;
use atty::Stream;
use clap::{ App, Arg };
use copypasta::{ ClipboardContext, ClipboardProvider };
use dotenv::dotenv;
use llm_low::*;
use std::io::{ self, Read };
use colored::*;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let mut ctx = ClipboardContext::new().expect("Failed to initialize clipboard");

    let matches = App::new("terminal-reader")
        .version("1.0")
        .arg(
            Arg::with_name("p")
                .long("prompt")
                .help("Specifies additional prompt")
                .takes_value(true)
        )
        .get_matches();

    let pat = matches.value_of("p");

    let mut input = String::new();
    if atty::is(atty::Stream::Stdin) {
        println!("No input piped in.");
    } else {
        io::stdin().read_to_string(&mut input)?;
    }
    let prompt_str = match pat {
        Some(pat) => format!("Here is what you need to pay attention to: `{:?}`", pat),
        None => String::new(),
    };

    let system_prompt = format!(
        "You're a coding bot, you're tasked to read terminal outputs and identify potential issues."
    );
    let user_input = format!(
        "These are the output of the program executed: `{input}`, {prompt_str} please identify the issues."
    );

    let model = "mistralai/Mistral-7B-Instruct-v0.1";
    let model = "Phind/Phind-CodeLlama-34B-v2";

    let res = chat_inner_async(&system_prompt, &input, 200, model).await?;
    ctx.set_contents(res.to_owned()).expect("Failed to set clipboard content");

    println!("{}:\n {}", "CodeLlama".color("yellow"), res.color("blue"));

    Ok(())
}
