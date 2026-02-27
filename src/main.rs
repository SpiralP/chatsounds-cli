#![warn(clippy::pedantic)]

mod setup;

use anyhow::Result;
use chatsounds::Chatsounds;
use clap::{Parser, Subcommand};

use self::setup::setup;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search for chatsounds matching the sentence
    Search { sentence: String },
    /// Play the chatsounds for the sentence
    #[cfg(feature = "playback")]
    Play { sentence: String },
    /// Render the chatsounds as raw PCM (f32le, 44100 Hz, stereo) to stdout
    #[cfg(feature = "render")]
    Render { sentence: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let chatsounds = setup().await?;

    match cli.command {
        Command::Search { sentence } => {
            search(&sentence, &chatsounds);
        }
        #[cfg(feature = "playback")]
        Command::Play { sentence } => {
            play_audio(&sentence, chatsounds).await?;
        }
        #[cfg(feature = "render")]
        Command::Render { sentence } => {
            use std::io::stdout;

            render_audio(&sentence, chatsounds, stdout().lock()).await?;
        }
    }

    Ok(())
}

fn search(input: &str, chatsounds: &Chatsounds) -> usize {
    let mut results = chatsounds.search(input);
    let results = results.drain(..).map(|(_, str)| str).collect::<Vec<_>>();
    for result in &results {
        println!("{result}");
    }

    results.len()
}

#[cfg(feature = "playback")]
async fn play_audio(input: &str, mut chatsounds: Chatsounds) -> Result<()> {
    use rand::rng;

    let (sink, _chatsounds) = chatsounds.play(input, rng()).await?;
    sink.sleep_until_end();

    Ok(())
}

#[cfg(feature = "render")]
async fn render_audio(
    input: &str,
    mut chatsounds: Chatsounds,
    writer: impl std::io::Write,
) -> Result<()> {
    use std::{
        io::{BufWriter, Write},
        num::NonZero,
    };

    use chatsounds::rodio::{Source, nz, queue::queue, source::UniformSourceIterator};
    use rand::rng;

    const SAMPLE_RATE: NonZero<u32> = nz!(44100);
    const CHANNELS: NonZero<u16> = nz!(2);
    const MAX_DURATION_SECS: u32 = 60;
    const MAX_SAMPLES: usize =
        (SAMPLE_RATE.get() * CHANNELS.get() as u32 * MAX_DURATION_SECS) as usize;

    let (mut sources, _chatsounds): (Vec<_>, Vec<_>) = chatsounds
        .get_sources(input, rng())
        .await?
        .into_iter()
        .unzip();

    eprintln!("{} sources", sources.len());

    if sources.is_empty() {
        search(input, &chatsounds);
        return Ok(());
    }

    let (sink, queue) = queue(false);
    for source in sources.drain(..) {
        sink.append(source);
    }
    let queue = UniformSourceIterator::new(queue, CHANNELS, SAMPLE_RATE);

    eprintln!(
        "{} Hz, {} channels, f32le (max {MAX_DURATION_SECS} sec)",
        queue.sample_rate(),
        queue.channels(),
    );

    let mut writer = BufWriter::new(writer);

    let mut queue = queue.peekable();
    for sample in queue.by_ref().take(MAX_SAMPLES) {
        writer.write_all(&sample.to_le_bytes())?;
    }

    if queue.peek().is_some() {
        eprintln!("warning: output truncated at {MAX_DURATION_SECS} seconds");
    }

    writer.flush()?;

    Ok(())
}

#[tokio::test]
async fn test_setup() {
    setup().await.unwrap();
}

#[tokio::test]
async fn test_search() {
    let chatsounds = setup().await.unwrap();
    let matches = search("ah hello gordon freeman its good to see you", &chatsounds);
    assert_eq!(matches, 1);
}

#[tokio::test]
#[cfg(feature = "render")]
async fn test_play_or_render_audio() {
    let chatsounds = setup().await.unwrap();

    let mut output = Vec::new();
    render_audio(
        "ah hello gordon freeman its good to see you",
        chatsounds,
        &mut output,
    )
    .await
    .unwrap();

    // f32le stereo 44100 Hz: 4 bytes per sample
    assert_eq!(output.len(), 1_225_632);
}

#[tokio::test]
#[cfg(feature = "render")]
async fn test_render_audio_truncates_loop() {
    let chatsounds = setup().await.unwrap();

    let mut output = Vec::new();
    render_audio(
        "ah hello gordon freeman its good to see you:loop()",
        chatsounds,
        &mut output,
    )
    .await
    .unwrap();

    // Should be truncated at MAX_SAMPLES (60 sec * 44100 Hz * 2 channels * 4 bytes)
    assert_eq!(output.len(), 60 * 44100 * 2 * 4);
}
