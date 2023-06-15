use crate::ctfd::{Challenge, ChallengeBrief, Ctfd};
use argh::FromArgs;
use chrono::{DateTime, Utc};
use color_eyre::{eyre, eyre::eyre, Report};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "snipe", description = "snipe a challenge")]
pub struct Snipe {
    #[argh(option, long = "flag")]
    /// the flag to submit
    flag: String,

    #[argh(option, long = "wait")]
    /// wait until this time
    wait_until: Option<chrono::DateTime<chrono::Utc>>,

    /// text to look for in title
    #[argh(positional, long = "title")]
    title_target: String,

    /// text to look for in the title and body
    #[argh(positional, long = "body")]
    body_target: Option<String>,
}

impl Snipe {
    pub async fn wait(&self, until: DateTime<Utc>) -> Result<(), Report> {
        use tokio::time::{sleep, Duration};

        let current_time = Utc::now();

        let diff = (until - current_time).to_std()?;

        let length = diff - Duration::from_secs(10);

        println!("Sleeping {}s", length.as_secs());
        sleep(length).await;

        println!("10 seconds left. spinlocking!");

        while Utc::now() < until {
            // ...
        }

        println!("Executing.");

        Ok(())
    }

    pub async fn challs(&self, ctf: &Ctfd) -> Result<Vec<ChallengeBrief>, Report> {
        use tokio::time::{sleep, Duration};

        let mut failed_cnt = 0;

        loop {
            let resp = ctf.get_challs().await;
            match resp {
                Ok(c) => return Ok(c),
                Err(e) => {
                    eprintln!("{:?}", e);

                    let base_length = Duration::from_millis(50);
                    //let factor = 2u32.pow(failed_cnt);
                    let factor = failed_cnt;
                    failed_cnt += 1;

                    let length = base_length * factor;

                    println!("Sleeping {:?}", length);

                    sleep(length).await;

                    continue;
                }
            }
        }
    }

    pub async fn find_title_matches(
        title: &str,
        challs: &[ChallengeBrief],
    ) -> Result<Vec<ChallengeBrief>, Report> {
        let title_search = title.to_lowercase();

        let matching_title: Vec<_> = challs
            .iter()
            .filter(|c| c.name.clone().to_lowercase().contains(&title_search))
            .cloned()
            .collect();

        Ok(matching_title)
    }

    pub async fn find_body_matches(
        body: &str,
        challs: &[Challenge],
    ) -> Result<Vec<Challenge>, Report> {
        let body_search = body.to_lowercase();

        let matching_body: Vec<_> = challs
            .iter()
            .filter(|c| c.description.clone().to_lowercase().contains(&body_search))
            .cloned()
            .collect();

        Ok(matching_body)
    }

    pub async fn submit(&self, id: i32) -> Result<(), Report> {
        todo!();
    }

    pub async fn run(&self, ctf: &Ctfd) -> Result<(), Report> {
        println!(
            "Main target '{}'. Secondary target '{:?}'",
            self.title_target, self.body_target
        );

        if let Some(until) = self.wait_until {
            self.wait(until).await?;
        } else {
            println!("Not waiting!");
        }

        let challs = self.challs(ctf).await?;
        dbg!(&challs);

        let title_matches = Self::find_title_matches(&self.title_target, &challs).await?;
        match title_matches.len() {
            0 => {}
            1 => {
                self.submit(title_matches[0].id).await?;
                return Ok(());
            }
            _ => {
                return Err(eyre!("Found multiple matches {:?}", title_matches));
            }
        }

        let body_target = if let Some(body_target) = self.body_target.as_ref() {
            body_target
        } else {
            // nothing else to try
            println!("Didn't find any matches");
            return Ok(());
        };

        // get bodies
        let full_challs = ctf.full_challs(&challs).await?;

        let body_matches = Self::find_body_matches(&body_target, &full_challs).await?;
        match body_matches.len() {
            0 => {}
            1 => {
                self.submit(body_matches[0].id).await?;
                return Ok(());
            }
            _ => {
                return Err(eyre!("Found multiple matches {:?}", body_matches));
            }
        }

        println!("Didn't find any matches");
        Ok(())
    }
}
