use maguro;
use tokio::fs::OpenOptions;

// Here we use the "current_thread" flavor since this is a simple
// application. In larger ones, a multithreaded runtime is preferable.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<(dyn error::Error + 'static)>> {
    // Get our video information and location the first format
    // available.
    let video_info = maguro::get_video_info("VfWgE7D1pYY").await?;
    let format = video_info.all_formats().first().cloned()?;

    // Open an asynchronous file handle.
    let mut output = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .open("maguro.mp4")
        .await?;

    // Download the video.
    println!("Downloading format:\n{}", format);
    format.download(&mut output).await?;
}
