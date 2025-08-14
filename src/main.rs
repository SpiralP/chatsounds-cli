mod setup;

use anyhow::{Context, Result};
use chatsounds::Chatsounds;
use rand::rng;

use self::setup::setup;

fn search(chatsounds: Chatsounds, input: &str) -> Result<usize> {
    let mut results = chatsounds.search(input);
    let results = results.drain(..).map(|(_, str)| str).collect::<Vec<_>>();
    println!("{:#?}", results);

    Ok(results.len())
}

#[tokio::main]
async fn main() -> Result<()> {
    let input = std::env::args().nth(1).context("need arg")?;

    let chatsounds = setup().await?;

    if input.starts_with("search ") {
        if let Some(input) = input.get("search ".len()..) {
            search(chatsounds, input)?;
            return Ok(());
        }
    }

    play_or_render_audio(&input, chatsounds).await?;

    Ok(())
}

async fn play_or_render_audio(input: &str, mut chatsounds: Chatsounds) -> Result<()> {
    #[cfg(feature = "playback")]
    {
        chatsounds
            .play(input, thread_rng())
            .await?
            .sleep_until_end();
    }

    #[cfg(not(feature = "playback"))]
    {
        use chatsounds::rodio::{source::UniformSourceIterator, Source};
        use hound::{SampleFormat, WavSpec, WavWriter};

        const OUT_FILE: &str = "output.wav";

        let (mut sources, _chatsounds): (Vec<_>, Vec<_>) = chatsounds
            .get_sources(input, rng())
            .await?
            .into_iter()
            .unzip();

        eprintln!("{} sources", sources.len());

        if sources.is_empty() {
            search(chatsounds, input)?;
            return Ok(());
        }

        let (sink, queue) = chatsounds::rodio::queue::queue(false);
        for source in sources.drain(..) {
            sink.append(source);
        }
        let queue = UniformSourceIterator::new(queue, 2, 44100);

        eprintln!("{} Hz, {} channels", queue.sample_rate(), queue.channels());

        let spec = WavSpec {
            channels: queue.channels(),
            sample_rate: queue.sample_rate(),
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        eprintln!("writing to {:?}", OUT_FILE);
        let mut writer = WavWriter::create(OUT_FILE, spec)?;

        for sample in queue {
            writer.write_sample(sample)?;
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_setup() {
    setup().await.unwrap();
}

#[tokio::test]
async fn test_search() {
    let chatsounds = setup().await.unwrap();
    let matches = search(chatsounds, "ah hello gordon freeman its good to see you").unwrap();
    assert_eq!(matches, 1);
}

#[tokio::test]
async fn test_play_or_render_audio() {
    let chatsounds = setup().await.unwrap();

    play_or_render_audio("ah hello gordon freeman its good to see you", chatsounds)
        .await
        .unwrap();
    let file = tokio::fs::File::open("output.wav").await.unwrap();
    assert_eq!(file.metadata().await.unwrap().len(), 1225372);
}
