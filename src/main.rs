use std::io::Cursor;
use std::path::PathBuf;

use argh::FromArgs;
use ctfd::Ctfd;
use reqwest::{self};
use reqwest::{Error, Url};

pub mod ctfd;
mod download;
use download::Download;
mod snipe;
use snipe::Snipe;

#[derive(FromArgs, Debug)]
/// Arguments
struct Args {
    /// domain to connect to
    #[argh(positional)]
    url: Url,

    /// session cookie
    #[argh(positional)]
    session: String,

    #[argh(subcommand)]
    cmd: Command,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Command {
    Download(Download),
    Snipe(Snipe),
}

impl Command {
    async fn run(&self, ctf: &Ctfd) -> Result<(), Error> {
        match self {
            Command::Download(d) => d.run(ctf).await,
            Command::Snipe(s) => s.run(ctf).await,
        }
    }
}

/// Data about a challenge
#[derive(Debug)]
pub struct Task<ID> {
    /// a unique id for this challenge
    /// used to correlate consequtive rusn
    _id: ID,
    /// name of the challenge
    /// used to make a folder for the challenge
    name: String,
    /// related downloads
    downloads: Vec<String>,
}

pub fn read_line_lower() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_ascii_lowercase()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let Args { url, session, cmd } = argh::from_env();

    // make sure we have a trailling slash, otherwise path is ignored: /asdf/blah + /endpoint => /endpoint
    // instead, have trailing slash and dont use prefixed slash: /asdf/blah/ + endpoint = /asdf/blah/endpoint
    let url: Url = {
        if url.as_str().ends_with('/') {
            url
        } else {
            (url.as_str().to_owned() + "/").parse().unwrap()
        }
    };

    let client = reqwest::Client::new();
    let ctf = Ctfd::new(client.clone(), url.clone(), format!("session={};", session));

    cmd.run(&ctf).await?;

    Ok(())
}
