use std::convert::TryInto;
use std::io::{BufReader, Read, Result};
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{AsyncRead, AsyncReadExt, Stream};

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


#[derive(Debug)]
pub struct AsyncEventStream<'a, F: AsRawFd + AsyncRead + Unpin> {
    file: &'a mut F,
    buffer: Vec<u8>,
    offset: usize,
}

impl<'a, F: AsRawFd + AsyncRead + Unpin> AsyncEventStream<'a, F> {
    pub(crate) fn from_device(device: &'a mut Device<F>) -> std::io::Result<Self> {
        Ok(AsyncEventStream { file: device.file_mut(), buffer: vec![0; 1024], offset: 0 })
    }
}

impl<'a, F: AsRawFd + AsyncRead + Unpin> AsyncEventStream<'a, F> {
    pub async fn read_next(&mut self) -> std::io::Result<Event> {
        const HEADER_LEN: usize = std::mem::size_of::<uapi::EventHeader>();

        while self.offset < HEADER_LEN {
            self.offset += self.file.read(&mut self.buffer[self.offset..]).await?;
        }

        let data_hdr = &self.buffer[..HEADER_LEN];
        let data_hdr: [u8; HEADER_LEN] = data_hdr.try_into().unwrap();
        let hdr: uapi::EventHeader = unsafe { std::mem::transmute_copy(&data_hdr) };

        let event_len = HEADER_LEN + hdr.length as usize;
        self.buffer.resize(event_len, 0);

        while self.offset < event_len {
            self.offset += self.file.read(&mut self.buffer[self.offset..]).await?;
        }
        self.offset = 0;

        Ok(Event {
            target_category: hdr.target_category,
            target_id: hdr.target_id,
            command_id: hdr.command_id,
            instance_id: hdr.instance_id,
            data: Vec::from(&self.buffer[HEADER_LEN..event_len]),
        })
    }
}

impl<'a, F: AsRawFd + AsyncRead + Unpin> Stream for AsyncEventStream<'a, F> {
    type Item = std::io::Result<Event>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        const HEADER_LEN: usize = std::mem::size_of::<uapi::EventHeader>();

        let s = Pin::into_inner(self);

        if s.offset < HEADER_LEN {
            s.offset += futures::ready!(Pin::new(&mut s.file)
                .poll_read(cx, &mut s.buffer[s.offset..]))?;
        }

        if s.offset < HEADER_LEN {
            return Poll::Pending;
        }

        let data_hdr = &s.buffer[..HEADER_LEN];
        let data_hdr: [u8; HEADER_LEN] = data_hdr.try_into().unwrap();
        let hdr: uapi::EventHeader = unsafe { std::mem::transmute_copy(&data_hdr) };

        let event_len = HEADER_LEN + hdr.length as usize;

        if s.offset < event_len {
            s.buffer.resize(event_len, 0);

            s.offset += futures::ready!(Pin::new(&mut s.file)
                .poll_read(cx, &mut s.buffer[s.offset..]))?;
        }

        if s.offset < event_len {
            return Poll::Pending;
        }

        s.offset = 0;

        let event = Event {
            target_category: hdr.target_category,
            target_id: hdr.target_id,
            command_id: hdr.command_id,
            instance_id: hdr.instance_id,
            data: Vec::from(&s.buffer[HEADER_LEN..event_len]),
        };
        Poll::Ready(Some(Ok(event)))
    }
}
