use std::io::Cursor;
use std::path::PathBuf;

use argh::FromArgs;
use ctfd::Ctfd;
use reqwest::{self};
use reqwest::{Error, Url};

pub mod ctfd;

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

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "download", description = "download all challenges")]
struct Download {
    // where to place challenges
    #[argh(positional)]
    out_path: PathBuf,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "snipe", description = "snipe a challenge")]
struct Snipe {
    /// keyword
    #[argh(positional)]
    target: String,

    #[argh(positional)]
    flag: String,
}

impl Command {
    async fn run(&self, ctf: &Ctfd) -> Result<(), Error> {
        match self {
            Command::Download(d) => d.run(ctf).await,
            Command::Snipe(_) => todo!(),
        }
    }
}

impl Download {
    async fn run(&self, ctf: &Ctfd) -> Result<(), Error> {
        let tasks = ctf.all_tasks().await?;

        // make sure path exists
        if !self.out_path.exists() {
            println!(
                "Diretory does not exist. Create '{}'? [Y/n]",
                self.out_path.display()
            );

            // read y or n
            let answer = read_line_lower();
            if answer == "y" || answer.is_empty() {
                std::fs::create_dir_all(&self.out_path).unwrap();
            } else {
                println!("Exiting...");
                return Ok(());
            }
        }

        // make sure there are no files there yet
        if self.out_path.read_dir().unwrap().count() > 0 {
            println!(
                "Directory '{}' is not empty. Continue? [Y/n]",
                self.out_path.display()
            );

            // read y or n
            let answer = read_line_lower();
            if answer != "y" && !answer.is_empty() {
                println!("Exiting...");
                return Ok(());
            }
        }

        for t in &tasks {
            let folder = t.name.to_lowercase().replace(' ', "_");
            let folder: String = folder
                .chars()
                .into_iter()
                .filter(|c| c.is_ascii_alphanumeric() || c == &'_')
                .collect();

            let path = self.out_path.join(folder);
            println!(
                "Placing challenge '{}' in folder '{}'",
                t.name,
                path.display()
            );

            if !path.exists() {
                std::fs::create_dir(path.clone()).unwrap();
            }

            for d in &t.downloads {
                let url: Url = format!("{base}{path}", base = ctf.base_url(), path = d)
                    .parse()
                    .unwrap();

                // get file name
                let file_name = url.path_segments().unwrap().last().unwrap();
                let file_path = path.join(file_name);
                //println!("Downloading to {}", file_path.display());

                if file_path.exists() {
                    println!(
                        "File '{}' already exists. Overwrite? [y/N]",
                        file_path.display()
                    );

                    let answer = read_line_lower();
                    if answer == "y" {
                        std::fs::create_dir_all(&self.out_path).unwrap();
                    } else {
                        // default: no
                        println!("Skipping");
                        continue;
                    }
                }
                let mut file = std::fs::File::create(file_path).expect("couldn't create file");

                let resp = ctf.client().get(url).send().await?;
                let mut content = Cursor::new(resp.bytes().await?);
                std::io::copy(&mut content, &mut file).expect("couldn't copy content");
            }
        }

        Ok(())
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

fn read_line_lower() -> String {
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
