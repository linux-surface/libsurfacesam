use nix::ioctl_readwrite;


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

ioctl_readwrite!(ssam_cdev_request, 0xa5, 0x01, Request);
