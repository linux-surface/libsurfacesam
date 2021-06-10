use nix::{ioctl_readwrite, ioctl_write_ptr};


pub const SSAM_CDEV_REQUEST_HAS_RESPONSE: u16 = 0x01;
pub const SSAM_CDEV_REQUEST_UNSEQUENCED: u16 = 0x02;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct RequestPayload {
    pub data: u64,
    pub length: u16,
    pub __pad: [u8; 6],
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct RequestResponse {
    pub data: u64,
    pub length: u16,
    pub __pad: [u8; 6],
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Request {
    pub target_category: u8,
    pub target_id: u8,
    pub command_id: u8,
    pub instance_id: u8,
    pub flags: u16,
    pub status: i16,
    pub payload: RequestPayload,
    pub response: RequestResponse,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct NotifierDesc {
    pub priority: i32,
    pub target_category: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EventHeader {
    pub target_category: u8,
    pub target_id: u8,
    pub command_id: u8,
    pub instance_id: u8,
    pub length: u16,
}

ioctl_readwrite!(ssam_cdev_request, 0xa5, 0x01, Request);
ioctl_write_ptr!(ssam_cdev_notif_register, 0xa5, 0x02, NotifierDesc);
ioctl_write_ptr!(ssam_cdev_notif_unregister, 0xa5, 0x03, NotifierDesc);
