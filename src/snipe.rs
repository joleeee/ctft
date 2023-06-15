use crate::ctfd::Ctfd;
use argh::FromArgs;
use reqwest::{self, Error};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "snipe", description = "snipe a challenge")]
pub struct Snipe {
    /// keyword
    #[argh(positional)]
    target: String,

    #[argh(positional)]
    flag: String,
}

impl Snipe {
    pub async fn run(&self, ctf: &Ctfd) -> Result<(), Error> {
        todo!()
    }
}
