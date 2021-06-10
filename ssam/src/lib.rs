use std::fs::File;
use std::io::ErrorKind;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub mod uapi;

pub use std::io::Error as Error;
pub use std::io::Result as Result;


#[derive(Debug)]
pub struct Request {
    pub target_category: u8,
    pub target_id: u8,
    pub command_id: u8,
    pub instance_id: u8,
    pub flags: u16,
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
        if payload.len() > std::u16::MAX as usize {
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
                data: if response.is_empty() { 0 } else { response.as_ptr() as u64 },
                length: response.len().min(std::u16::MAX as usize) as u16,
                __pad: [0; 6],
            },
        };

        unsafe { uapi::ssam_cdev_request(self.file.as_raw_fd(), &mut rqst as *mut _) }
            .map_err(nix_to_io_err)?;

        if rqst.status >= 0 {
            Ok(rqst.response.length as usize)
        } else {
            Err(Error::from_raw_os_error(rqst.status as i32))
        }
    }
}

impl<F> From<F> for Device<F> {
    fn from(file: F) -> Self {
        Self::new(file)
    }
}


fn nix_to_io_err(err: nix::Error) -> std::io::Error {
    match err {
        nix::Error::Sys(errno)           => Error::from_raw_os_error(errno as i32),
        nix::Error::InvalidPath          => Error::new(ErrorKind::InvalidInput, err),
        nix::Error::InvalidUtf8          => Error::new(ErrorKind::InvalidData, err),
        nix::Error::UnsupportedOperation => Error::new(ErrorKind::Other, err),
    }
}
