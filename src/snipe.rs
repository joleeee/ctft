use crate::ctfd::{ChallengeBrief, Ctfd};
use argh::FromArgs;
use chrono::{DateTime, Utc};
use color_eyre::Report;

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "snipe", description = "snipe a challenge")]
pub struct Snipe {
    /// keyword
    #[argh(positional)]
    target: String,

    #[argh(positional)]
    flag: String,

    /// wait until this time
    #[argh(positional)]
    wait_until: Option<chrono::DateTime<chrono::Utc>>,
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

    pub async fn run(&self, ctf: &Ctfd) -> Result<(), Report> {
        if let Some(until) = self.wait_until {
            self.wait(until).await?;
        } else {
            println!("Not waiting!");
        }

        let challs = self.challs(ctf).await?;
        dbg!(&challs);

        // first, find the challenge
        todo!()
    }
}
