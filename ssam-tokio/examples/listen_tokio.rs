use std::io::Result;

use futures::StreamExt;


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut device = ssam_tokio::connect().await?;

    // We assume that battery events (category 0x02) are already enabled...
    // If not, they should be enabled via device.event_enable(...) here.
    device.notifier_register(0x02, 0)?;

    let mut events = device.events_async()?;
    while let Some(event) = events.next().await {
        println!("{:?}", event?);
    }

    Ok(())
}
