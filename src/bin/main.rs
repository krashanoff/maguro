use clap::clap_app;
use log::{error, info, LevelFilter};
use std::{error, process::exit};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use maguro;

mod maglog {
    use chrono::Utc;
    use log::{max_level, Log, Metadata, Record};

    pub struct MagnetLogger;

    impl Log for MagnetLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= max_level()
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                println!("{} - {} - {}", Utc::now(), record.level(), record.args());
            }
        }

        fn flush(&self) {}
    }
}

static LOGGER: maglog::MagnetLogger = maglog::MagnetLogger;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<(dyn error::Error + 'static)>> {
    let matches = clap_app!(maguro =>
        (version: "0.0.1")
        (author: "krashanoff <leo@krashanoff.com>")
        (about: "A fast YouTube downloader.")
        (@arg verbose: -v ... "Increases program verbosity")
        (@arg show_formats: -F --formats "Display formats available for download and exit")
        (@arg format: -f +takes_value "Downloads a specific format by `itag`. Defaults to highest quality.")
        (@arg output: -o --output +takes_value +required "Outputs videos with the given file extension")
        (@arg VIDEOS: ... +required "Video(s) to download or introspect on")
    )
    .get_matches();

    if let Err(e) = log::set_logger(&LOGGER).map(|()| {
        log::set_max_level(match matches.occurrences_of("verbose") {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            _ => LevelFilter::Debug,
        })
    }) {
        panic!("Failed to initialize logger! {}", e)
    }

    // Get video and output ID information.
    let mut ids: Vec<&str> = matches
        .values_of("VIDEOS")
        .unwrap_or_else(|| panic!("A list of video IDs is required!"))
        .collect();
    let ext = matches.value_of("output").unwrap();

    let mut info: Vec<maguro::InfoResponse> = Vec::new();
    while let Some(id) = ids.pop() {
        info!("Collecting data for {}", id);
        let vid_info = maguro::get_video_info(id).await?;
        info.push(vid_info);
    }

    // Outputs available formats then exits.
    if matches.is_present("show_formats") {
        for resp in info {
            println!(
                "Displaying available formats for video ID {}:",
                resp.details().id()
            );
            for format in &resp.all_formats() {
                println!("{}", format);
            }
        }
        exit(0)
    }

    // Otherwise, download videos.
    for resp in info {
        println!("Starting download of {}...", resp.details().id());

        let mut dest = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .open(format!("{}.{}", resp.details().id(), ext))
            .await?;

        let formats = resp.all_formats();
        let chosen = match matches.value_of("format") {
            Some(fmt) => formats.iter().find(|&f| f.itag().to_string() == fmt),
            None => formats.last(),
        };

        let vid = match chosen {
            Some(f) => f
                .to_vec_callback(|b| {
                    info!("Downloaded {} bytes...", b.len());
                    Ok(())
                })
                .await
                .unwrap(),
            None => {
                error!("Failed to find selected itag!");
                exit(1)
            }
        };

        info!(
            "Final vector is of length {}/{:?}",
            &vid.len(),
            &chosen.unwrap().size()
        );
        dest.write_all(vid.as_slice()).await?;
        println!("Completed download of video {}.", resp.details().id());
    }

    Ok(())
}
