mod error;

use crate::error::*;
use chatsounds::{Chatsounds, Source};
use rand::thread_rng;

enum LoadSource {
    Api(&'static str, &'static str),
    Msgpack(&'static str, &'static str),
}

const SOURCES: &[LoadSource] = &[
    LoadSource::Api("NotAwesome2/chatsounds", "sounds"),
    LoadSource::Api(
        "Metastruct/garrysmod-chatsounds",
        "sound/chatsounds/autoadd",
    ),
    LoadSource::Api("PAC3-Server/chatsounds", "sounds/chatsounds"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "csgo"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "css"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "ep1"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "ep2"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "hl1"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "hl2"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d2"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "portal"),
    LoadSource::Msgpack("PAC3-Server/chatsounds-valve-games", "tf2"),
];

async fn setup() -> Chatsounds {
    tokio::fs::create_dir_all("chatsounds").await.unwrap();

    let mut chatsounds = Chatsounds::new("chatsounds").unwrap();

    let sources_len = SOURCES.len();
    for (i, source) in SOURCES.iter().enumerate() {
        let (repo, repo_path) = match source {
            LoadSource::Api(repo, repo_path) | LoadSource::Msgpack(repo, repo_path) => {
                (repo, repo_path)
            }
        };

        println!(
            "[{}/{}] fetching {} {}",
            i + 1,
            sources_len,
            repo,
            repo_path
        );

        match source {
            LoadSource::Api(repo, repo_path) => {
                chatsounds.load_github_api(repo, repo_path).await.unwrap()
            }
            LoadSource::Msgpack(repo, repo_path) => chatsounds
                .load_github_msgpack(repo, repo_path)
                .await
                .unwrap(),
        }
    }

    chatsounds
}

#[tokio::main]
async fn main() -> Result<()> {
    let input = std::env::args().nth(1).unwrap();

    let mut chatsounds = setup().await;

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
