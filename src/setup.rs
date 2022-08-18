use std::path::PathBuf;

use anyhow::Result;
use chatsounds::Chatsounds;
use futures::{FutureExt, StreamExt, TryFutureExt};
use tokio::fs;

struct GitHubRepo {
    name: &'static str,
    path: &'static str,
}

enum Source {
    Api(GitHubRepo),
    MsgPack(GitHubRepo),
}
impl Source {
    pub const fn api(name: &'static str, path: &'static str) -> Self {
        Source::Api(GitHubRepo { name, path })
    }

    pub const fn msgpack(name: &'static str, path: &'static str) -> Self {
        Source::MsgPack(GitHubRepo { name, path })
    }
}

const SOURCES: &[Source] = &[
    Source::api("NotAwesome2/chatsounds", "sounds"),
    Source::api(
        "Metastruct/garrysmod-chatsounds",
        "sound/chatsounds/autoadd",
    ),
    Source::api("PAC3-Server/chatsounds", "sounds/chatsounds"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "csgo"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "css"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "ep1"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "ep2"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "hl1"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "hl2"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "l4d"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "l4d2"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "portal"),
    Source::msgpack("PAC3-Server/chatsounds-valve-games", "tf2"),
];

async fn load_sources(chatsounds: &mut Chatsounds) -> Result<()> {
    enum SourceData {
        Api(chatsounds::GitHubApiTrees),
        MsgPack(chatsounds::GitHubMsgpackEntries),
    }

    let stream = futures::stream::iter(SOURCES)
        .map(|source| match source {
            Source::Api(repo) => chatsounds
                .fetch_github_api(repo.name, repo.path)
                .map_ok(move |data| (repo, SourceData::Api(data)))
                .boxed(),

            Source::MsgPack(repo) => chatsounds
                .fetch_github_msgpack(repo.name, repo.path)
                .map_ok(move |data| (repo, SourceData::MsgPack(data)))
                .boxed(),
        })
        .buffer_unordered(8);

    let results = stream.collect::<Vec<std::result::Result<_, _>>>().await;

    for result in results {
        match result {
            Err(e) => {
                eprintln!("Failed to fetch: {:?}", e);
            }

            Ok((repo, data)) => match data {
                SourceData::Api(data) => chatsounds.load_github_api(repo.name, repo.path, data)?,
                SourceData::MsgPack(data) => {
                    chatsounds.load_github_msgpack(repo.name, repo.path, data)?
                }
            },
        }
    }

    Ok(())
}

pub async fn setup() -> Result<Chatsounds> {
    let cache_dir = PathBuf::from("chatsounds");
    fs::create_dir_all(&cache_dir).await?;

    let mut chatsounds = Chatsounds::new(&cache_dir)?;
    load_sources(&mut chatsounds).await?;

    Ok(chatsounds)
}
