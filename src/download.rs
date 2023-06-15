use color_eyre::eyre::Report;
use std::io::Cursor;
use std::path::PathBuf;

use crate::ctfd::Ctfd;
use crate::read_line_lower;
use argh::FromArgs;
use reqwest::{self, Url};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "download", description = "download all challenges")]
pub struct Download {
    // where to place challenges
    #[argh(positional)]
    out_path: PathBuf,

    #[argh(switch, short = 'f')]
    /// say yes
    force: bool,
}

impl Download {
    fn make_path(&self) -> Result<(), Report> {
        if self.out_path.exists() {
            return Ok(());
        }

        std::fs::create_dir_all(&self.out_path)?;

        Ok(())
    }

    fn root_write_approval(&self) -> Result<Option<()>, Report> {
        let file_count = self.out_path.read_dir()?.count();
        if file_count == 0 {
            return Ok(Some(()));
        }

        let approval = self.force || {
            println!(
                "Directory '{}' is not empty. Continue? [Y/n]",
                self.out_path.display()
            );

            // read y or n
            let answer = read_line_lower();
            answer == "y" || answer.is_empty()
        };

        if approval {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    fn sanitize_name(name: &str) -> String {
        name.replace(' ', "_")
            .chars()
            .into_iter()
            .filter(|c| c.is_ascii_alphanumeric() || c == &'_')
            .collect()
    }

    fn file_write_approval(&self, path: &PathBuf) -> Result<bool, Report> {
        if !path.exists() {
            // no file there, can safely write
            return Ok(true);
        }

        let approval = self.force || {
            println!("File '{}' already exists. Overwrite? [y/N]", path.display());

            let answer = read_line_lower();
            answer == "y"
        };

        Ok(approval)
    }

    pub async fn run(&self, ctf: &Ctfd) -> Result<(), Report> {
        self.make_path()?;

        if self.root_write_approval()?.is_none() {
            // early exit, no write approval
            return Ok(());
        }

        let tasks = ctf.all_tasks().await?;

        for t in &tasks {
            let folder = Self::sanitize_name(&t.name);

            let path = self.out_path.join(folder);
            println!(
                "Placing challenge '{}' in folder '{}'",
                t.name,
                path.display()
            );

            if !path.exists() {
                std::fs::create_dir(path.clone())?;
            }

            for d in &t.downloads {
                let url: Url = format!("{base}{path}", base = ctf.base_url(), path = d).parse()?;

                // get file name
                let file_name = url.path_segments().unwrap().last().unwrap();
                let file_path = path.join(file_name);

                let can_write = self.file_write_approval(&file_path)?;

                if !can_write {
                    println!("Skipping...");
                    continue;
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
