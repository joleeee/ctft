use std::io::Cursor;
use std::path::PathBuf;

use crate::ctfd::Ctfd;
use crate::read_line_lower;
use argh::FromArgs;
use reqwest::{self};
use reqwest::{Error, Url};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "download", description = "download all challenges")]
pub struct Download {
    // where to place challenges
    #[argh(positional)]
    out_path: PathBuf,
}

impl Download {
    pub async fn run(&self, ctf: &Ctfd) -> Result<(), Error> {
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
