fn main() -> ssam::Result<()> {
    let mut device = ssam::connect()?;

    // We assume that battery events (category 0x02) are already enabled...
    // If not, they should be enabled via device.event_enable(...) here.

    device.notifier_register(0x02, 0)?;
    for event in device.events()? {
        println!("{:?}", event?);
    }

    Ok(())
}
