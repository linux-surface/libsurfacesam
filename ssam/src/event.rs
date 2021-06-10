use std::io::{BufReader, Read, Result};
use std::os::unix::io::AsRawFd;

use crate::Device;
use crate::uapi;


#[derive(Debug, Clone)]
pub struct Event {
    pub target_category: u8,
    pub target_id: u8,
    pub command_id: u8,
    pub instance_id: u8,
    pub data: Vec<u8>,
}


pub struct EventStream<'a, F: AsRawFd> {
    reader: BufReader<&'a mut F>,
}

impl<'a, F: AsRawFd + Read> EventStream<'a, F> {
    pub(crate) fn from_device(device: &'a mut Device<F>) -> Result<Self> {
        let reader = BufReader::with_capacity(1024, device.file_mut());
        Ok(EventStream { reader })
    }
}

impl<'a, F: AsRawFd + Read> EventStream<'a, F> {
    pub fn read_next_blocking(&mut self) -> Result<Event> {
        let mut buf_hdr = [0; std::mem::size_of::<uapi::EventHeader>()];
        let mut buf_data = Vec::new();

        self.reader.read_exact(&mut buf_hdr)?;

        let hdr: uapi::EventHeader = unsafe { std::mem::transmute_copy(&buf_hdr) };

        buf_data.resize(hdr.length as usize, 0);
        self.reader.read_exact(&mut buf_data)?;

        Ok(Event {
            target_category: hdr.target_category,
            target_id: hdr.target_id,
            command_id: hdr.command_id,
            instance_id: hdr.instance_id,
            data: buf_data,
        })
    }
}

impl<'a, F: AsRawFd + Read> Iterator for EventStream<'a, F> {
    type Item = Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.read_next_blocking())
    }
}
