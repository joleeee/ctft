use crate::ctfd::Ctfd;
use argh::FromArgs;
use color_eyre::Report;
use reqwest::{self};

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
    pub async fn run(&self, ctf: &Ctfd) -> Result<(), Report> {
        todo!()
    }
}
