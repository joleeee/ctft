use std::fmt;

use reqwest::Url;
use reqwest::{self};
use serde::{de::DeserializeOwned, Deserialize};

use crate::Task;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[serde(bound = "D: DeserializeOwned")]
enum ApiResult<D: DeserializeOwned> {
    Ok(ApiResponse<D>),
    Err(ApiError),
}

impl<D> ApiResult<D>
where
    for<'de> D: Deserialize<'de>,
{
    fn result(self) -> Result<ApiResponse<D>, ApiError> {
        match self {
            ApiResult::Ok(ok) => Ok(ok),
            ApiResult::Err(err) => Err(err),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(bound = "D: DeserializeOwned")]
pub struct ApiResponse<D: DeserializeOwned> {
    pub success: bool,
    pub data: D,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiError {
    message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"Server returned message "{}""#, self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(thiserror::Error, Debug)]
pub enum CtfdError {
    #[error("ApiError")]
    ApiError(#[from] ApiError),
    #[error("Reqwest")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChallengeBrief {
    pub name: String,
    pub id: i32,
    pub category: String,
    pub value: i32,
    pub solved_by_me: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Challenge {
    pub name: String,
    pub id: i32,

    pub category: String,
    pub description: String,
    pub connection_info: Option<String>,

    pub value: i32,
    pub solved_by_me: bool,

    pub files: Vec<String>,
}

impl Ctfd {
    pub async fn get_challs(&self) -> Result<Vec<ChallengeBrief>, CtfdError> {
        let url = self.base_url.join("api/v1/challenges").unwrap();

        let resp = self
            .client
            .get(url)
            .header("cookie", &self.session)
            .send()
            .await?;

        let vec: ApiResponse<Vec<ChallengeBrief>> = resp.json::<ApiResult<_>>().await?.result()?;

        if vec.success {
            Ok(vec.data)
        } else {
            panic!();
        }
    }

    pub async fn get_chall(&self, id: i32) -> Result<Challenge, CtfdError> {
        let url = self
            .base_url
            .join(&format!("api/v1/challenges/{id}"))
            .unwrap();

        let resp = self
            .client
            .get(url)
            .header("cookie", &self.session)
            .send()
            .await
            .unwrap();

        let chal: ApiResponse<Challenge> = resp.json().await?;

        if chal.success {
            Ok(chal.data)
        } else {
            panic!();
        }
    }

    pub async fn all_tasks(&self) -> Result<Vec<Task<i32>>, CtfdError> {
        let chal_ids = self
            .get_challs()
            .await?
            .iter()
            .map(|v| v.id)
            .collect::<Vec<_>>();

        let mut tasks = Vec::new();
        for id in chal_ids {
            let chal = self.get_chall(id).await?;
            assert_eq!(id, chal.id);
            tasks.push(Task {
                _id: chal.id,
                name: chal.name,
                downloads: chal.files,
            });
        }

        Ok(tasks)
    }
}

pub struct Ctfd {
    client: reqwest::Client,
    base_url: Url,
    session: String,
}

impl Ctfd {
    pub fn new(client: reqwest::Client, base_url: Url, session: String) -> Self {
        Ctfd {
            client,
            base_url,
            session,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url[..]
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
