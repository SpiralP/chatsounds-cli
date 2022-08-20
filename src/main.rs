mod setup;

use anyhow::{Context, Result};
use chatsounds::Chatsounds;
use hound::{SampleFormat, WavSpec, WavWriter};
use rand::thread_rng;
use rodio::source::UniformSourceIterator;

use self::setup::setup;

fn search(chatsounds: Chatsounds, input: &str) -> Result<()> {
    let mut results = chatsounds.search(input);
    let results = results.drain(..).map(|(_, str)| str).collect::<Vec<_>>();
    println!("{:#?}", results);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let input = std::env::args().nth(1).context("need arg")?;

    let mut chatsounds = setup().await?;

    if input.starts_with("search ") {
        if let Some(input) = input.get("search ".len()..) {
            search(chatsounds, input)?;
            return Ok(());
        }
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

        const OUT_FILE: &str = "output.wav";

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
        eprintln!("writing to {:?}", OUT_FILE);
        let mut writer = WavWriter::create(OUT_FILE, spec)?;

        for sample in queue {
            writer.write_sample(sample)?;
        }
    }

    Ok(())
}
