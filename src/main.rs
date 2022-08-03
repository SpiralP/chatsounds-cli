use anyhow::{Context, Result};
use chatsounds::Chatsounds;
use futures::prelude::*;
use hound::{SampleFormat, WavSpec, WavWriter};
use rand::thread_rng;
use rodio::source::UniformSourceIterator;

struct GitHubRepo {
    name: &'static str,
    path: &'static str,
}

enum Source {
    Api(GitHubRepo),
    MsgPack(GitHubRepo),
}
impl Source {
    const fn api(name: &'static str, path: &'static str) -> Source {
        Source::Api(GitHubRepo { name, path })
    }

    const fn msgpack(name: &'static str, path: &'static str) -> Source {
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
                .fetch_github_api(repo.name, repo.path, true)
                .map_ok(move |data| (repo, SourceData::Api(data)))
                .boxed(),

            Source::MsgPack(repo) => chatsounds
                .fetch_github_msgpack(repo.name, repo.path, true)
                .map_ok(move |data| (repo, SourceData::MsgPack(data)))
                .boxed(),
        })
        .buffered(5);

    let fetched = stream.try_collect::<Vec<_>>().await?;

    for (repo, data) in fetched {
        match data {
            SourceData::Api(data) => chatsounds.load_github_api(repo.name, repo.path, data)?,
            SourceData::MsgPack(data) => {
                chatsounds.load_github_msgpack(repo.name, repo.path, data)?
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let input = std::env::args().nth(1).context("need arg")?;

    let cache_dir = "chatsounds";
    tokio::fs::create_dir_all(cache_dir).await?;

    let mut chatsounds = Chatsounds::new(cache_dir)?;
    load_sources(&mut chatsounds).await?;

    let search_input = input.trim_start_matches("search ");
    if !search_input.is_empty() {
        search(chatsounds, search_input)?;
        return Ok(());
    }

    #[cfg(feature = "playback")]
    {
        chatsounds
            .play(&input, thread_rng())
            .await?
            .sleep_until_end();
    }

    #[cfg(not(feature = "playback"))]
    {
        use chatsounds::Source;

        let mut sources = chatsounds.get_sources(&input, thread_rng()).await?;

        eprintln!("{} sources", sources.len());

        if sources.is_empty() {
            search(chatsounds, &input)?;
            return Ok(());
        }

        let (sink, queue) = rodio::queue::queue(false);
        for source in sources.drain(..) {
            sink.append(source);
        }
        let queue: UniformSourceIterator<_, i16> = UniformSourceIterator::new(queue, 2, 44100);

        eprintln!("{} Hz, {} channels", queue.sample_rate(), queue.channels());

        let spec = WavSpec {
            channels: queue.channels(),
            sample_rate: queue.sample_rate(),
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        eprintln!("writing to output.wav");
        let mut writer = WavWriter::create("output.wav", spec)?;

        for sample in queue {
            writer.write_sample(sample)?;
        }
    }

    Ok(())
}

fn search(chatsounds: Chatsounds, input: &str) -> Result<()> {
    let mut results = chatsounds.search(&input);
    let results = results.drain(..).map(|(_, str)| str).collect::<Vec<_>>();
    println!("{:#?}", results);

    Ok(())
}
