use std::fs::File;
use std::io::{ErrorKind, Read};
use std::os::unix::io::AsRawFd;
use std::path::Path;

use futures::AsyncRead;

use tracing::trace;

pub mod uapi;

pub mod event;
pub use event::{Event, EventStream, AsyncEventStream};

pub use std::io::Error as Error;
pub use std::io::Result as Result;


#[derive(Debug, Clone, Copy)]
pub struct Request {
    pub target_category: u8,
    pub target_id: u8,
    pub command_id: u8,
    pub instance_id: u8,
    pub flags: u16,
}


#[derive(Debug, Clone, Copy)]
pub struct EventRegistry {
    pub target_category: u8,
    pub target_id: u8,
    pub cid_enable: u8,
    pub cid_disable: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct EventId {
    pub target_category: u8,
    pub instance: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct EventDescriptor {
    pub reg: EventRegistry,
    pub id: EventId,
    pub flags: u8,
}


impl From<&EventRegistry> for uapi::EventRegistry {
    fn from(reg: &EventRegistry) -> Self {
        uapi::EventRegistry {
            target_category: reg.target_category,
            target_id: reg.target_id,
            cid_enable: reg.cid_enable,
            cid_disable: reg.cid_disable,
        }
    }
}

impl From<&EventId> for uapi::EventId {
    fn from(id: &EventId) -> Self {
        uapi::EventId {
            target_category: id.target_category,
            instance: id.instance,
        }
    }
}

impl From<&EventDescriptor> for uapi::EventDesc {
    fn from(desc: &EventDescriptor) -> Self {
        uapi::EventDesc {
            reg: uapi::EventRegistry::from(&desc.reg),
            id: uapi::EventId::from(&desc.id),
            flags: desc.flags,
        }
    }
}


pub const DEFAULT_DEVICE_FILE_PATH: &str = "/dev/surface/aggregator";

pub fn connect() -> Result<Device<File>> {
    Device::open()
}


#[derive(Debug)]
pub struct Device<F> {
    file: F,
}

impl<F> Device<F> {
    fn new(file: F) -> Self {
        Device { file }
    }

    pub fn file(&self) -> &F {
        &self.file
    }

    pub fn file_mut(&mut self) -> &mut F {
        &mut self.file
    }
}

impl Device<File> {
    pub fn open() -> Result<Self> {
        Device::open_path(DEFAULT_DEVICE_FILE_PATH)
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Device {
            file: File::open(path)?,
        })
    }
}

impl<F: AsRawFd> Device<F> {
    pub fn request(&self, desc: &Request, payload: &[u8], response: &mut [u8]) -> Result<usize> {
        if payload.len() > u16::MAX as usize {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        let mut rqst = uapi::Request {
            target_category: desc.target_category,
            target_id: desc.target_id,
            command_id: desc.command_id,
            instance_id: desc.instance_id,
            flags: desc.flags,
            status: 0,
            payload: uapi::RequestPayload {
                data: if payload.is_empty() { 0 } else { payload.as_ptr() as u64 },
                length: payload.len() as u16,
                __pad: [0; 6],
            },
            response: uapi::RequestResponse {
                data: if response.is_empty() { 0 } else { response.as_mut_ptr() as u64 },
                length: response.len().min(u16::MAX as usize) as u16,
                __pad: [0; 6],
            },
        };

        let result = unsafe { uapi::ssam_cdev_request(self.file.as_raw_fd(), &mut rqst as *mut _) }
            .map(|_| ());

        let status = rqst.status as i32;
        match result {
            Ok(()) => trace!(target: "ssam::ioctl", status=%status, "ssam_cdev_request"),
            Err(ref e) => trace!(target: "ssam::ioctl", error=%e, status=%status, "ssam_cdev_request"),
        }

        if status >= 0 {
            Ok(rqst.response.length as usize)
        } else {
            Err(Error::from_raw_os_error(status))
        }
    }

    pub fn notifier_register(&self, target_category: u8, priority: i32) -> Result<()> {
        let desc = uapi::NotifierDesc { priority, target_category };

        let result = unsafe { uapi::ssam_cdev_notif_register(self.file.as_raw_fd(), &desc as *const _) }
            .map(|_| ());

        match result {
            Ok(()) => trace!(target: "ssam::ioctl", "ssam_cdev_notif_register"),
            Err(ref e) => trace!(target: "ssam::ioctl", error=%e, "ssam_cdev_notif_register"),
        }

        Ok(result?)
    }

    pub fn notifier_unregister(&self, target_category: u8) -> Result<()> {
        let desc = uapi::NotifierDesc { priority: 0 /* ignored */, target_category };

        let result = unsafe { uapi::ssam_cdev_notif_unregister(self.file.as_raw_fd(), &desc as *const _) }
            .map(|_| ());

        match result {
            Ok(()) => trace!(target: "ssam::ioctl", "ssam_cdev_notif_unregister"),
            Err(ref e) => trace!(target: "ssam::ioctl", error=%e, "ssam_cdev_notif_unregister"),
        }

        Ok(result?)
    }

    pub fn event_enable(&self, desc: &EventDescriptor) -> Result<()> {
        let d = uapi::EventDesc::from(desc);

        let result = unsafe { uapi::ssam_cdev_event_enable(self.file.as_raw_fd(), &d as *const _) }
            .map(|_| ());

        match result {
            Ok(()) => trace!(target: "ssam::ioctl", "ssam_cdev_event_enable"),
            Err(ref e) => trace!(target: "ssam::ioctl", error=%e, "ssam_cdev_event_enable"),
        }

        Ok(result?)
    }

    pub fn event_disable(&self, desc: &EventDescriptor) -> Result<()> {
        let d = uapi::EventDesc::from(desc);

        let result = unsafe { uapi::ssam_cdev_event_disable(self.file.as_raw_fd(), &d as *const _) }
            .map(|_| ());

        match result {
            Ok(()) => trace!(target: "ssam::ioctl", "ssam_cdev_event_disable"),
            Err(ref e) => trace!(target: "ssam::ioctl", error=%e, "ssam_cdev_event_disable"),
        }

        Ok(result?)
    }
}

impl<F: AsRawFd + Read> Device<F> {
    pub fn events(&mut self) -> std::io::Result<EventStream<F>> {
        EventStream::from_device(self)
    }
}

impl<F: AsRawFd + AsyncRead + Unpin> Device<F> {
    pub fn events_async(&mut self) -> std::io::Result<AsyncEventStream<F>> {
        AsyncEventStream::from_device(self)
    }
}

impl<F> From<F> for Device<F> {
    fn from(file: F) -> Self {
        Self::new(file)
    }
}
