use ssam::Request;

fn main() -> ssam::Result<()> {
	let request = Request {
        target_category: 0x01,
        target_id: 0x01,
        command_id: 0x13,
        instance_id: 0x00,
        flags: ssam::uapi::SSAM_CDEV_REQUEST_HAS_RESPONSE,
    };

    let mut response: [u8; 4] = [0; 4];

    ssam::connect()?
        .request(&request, &[], &mut response)?;

    println!("{}.{}.{}", response[3], ((response[2] as u16) << 8) | (response[1] as u16), response[0]);

    Ok(())
}
