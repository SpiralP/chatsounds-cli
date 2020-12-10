mod error;

use crate::error::*;
use chatsounds::Chatsounds;
use futures::prelude::*;
use rand::thread_rng;

#[derive(Copy, Clone)]
enum SourceKind {
    Api,
    Msgpack,
}

#[derive(Copy, Clone)]
struct Source {
    repo: &'static str,
    repo_path: &'static str,
    kind: SourceKind,
}
impl Source {
    const fn api(repo: &'static str, repo_path: &'static str) -> Self {
        Self {
            repo,
            repo_path,
            kind: SourceKind::Api,
        }
    }

    const fn msgpack(repo: &'static str, repo_path: &'static str) -> Self {
        Self {
            repo,
            repo_path,
            kind: SourceKind::Msgpack,
        }
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

async fn load_sources(chatsounds: &mut Chatsounds) {
    enum FetchedSource {
        Api(chatsounds::GitHubApiTrees),
        Msgpack(chatsounds::GitHubMsgpackEntries),
    }

    // TODO undo this weirdness when this is fixed
    // https://github.com/rust-lang/rust/issues/64552#issuecomment-669728225
    let stream: std::pin::Pin<Box<dyn Stream<Item = _> + Send>> = Box::pin(
        futures::stream::iter(SOURCES)
            .map(
                |Source {
                     repo,
                     repo_path,
                     kind,
                 }| {
                    match kind {
                        SourceKind::Api => chatsounds
                            .fetch_github_api(repo, repo_path, true)
                            .map_ok(FetchedSource::Api)
                            .boxed(),

                        SourceKind::Msgpack => chatsounds
                            .fetch_github_msgpack(repo, repo_path, true)
                            .map_ok(FetchedSource::Msgpack)
                            .boxed(),
                    }
                    .map_ok(move |fetched_source| (*repo, *repo_path, fetched_source))
                },
            )
            .buffered(5),
    );

    let fetched = stream.try_collect::<Vec<_>>().await.unwrap();

    for (repo, repo_path, fetched_source) in fetched {
        match fetched_source {
            FetchedSource::Api(data) => {
                chatsounds.load_github_api(repo, repo_path, data).unwrap();
            }

            FetchedSource::Msgpack(data) => {
                chatsounds
                    .load_github_msgpack(repo, repo_path, data)
                    .unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    use chatsounds::Source;

    let input = std::env::args().nth(1).unwrap();

    tokio::fs::create_dir_all("chatsounds").await.unwrap();

    let mut chatsounds = Chatsounds::new("chatsounds").unwrap();

    load_sources(&mut chatsounds).await;

    #[cfg(feature = "playback")]
    chatsounds
        .play(input, thread_rng())
        .await
        .unwrap()
        .sleep_until_end();

    #[cfg(not(feature = "playback"))]
    {
        let queue = chatsounds
            .get_sources_queue(input, thread_rng())
            .await
            .unwrap();

        println!("{} Hz, {} channels", queue.sample_rate(), queue.channels());

        let spec = hound::WavSpec {
            channels: queue.channels(),
            sample_rate: queue.sample_rate(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        println!("writing to output.wav");
        let mut writer = hound::WavWriter::create("output.wav", spec).unwrap();

        for sample in queue {
            writer.write_sample(sample).unwrap();
        }
    }

    Ok(())
}
