//! Limine Bootloader Protocol Interface (Specification v7)
//!
//! Declares the memory structures and magic requests required by the Limine bootloader
//! to boot the Yashwanth Micro-OS x86_64 ELF binary.

#[repr(C)]
pub struct LimineBaseRevision {
    pub id: [u64; 2],
    pub revision: u64,
}

#[repr(C)]
pub struct LimineHhdmResponse {
    pub revision: u64,
    pub offset: u64,
}

#[repr(C)]
pub struct LimineHhdmRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineHhdmResponse,
}

unsafe impl Sync for LimineHhdmRequest {}

#[repr(C)]
pub struct LimineMemmapEntry {
    pub base: u64,
    pub length: u64,
    pub entry_type: u64,
}

#[repr(C)]
pub struct LimineMemmapResponse {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *const *const LimineMemmapEntry,
}

#[repr(C)]
pub struct LimineMemmapRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineMemmapResponse,
}

unsafe impl Sync for LimineMemmapRequest {}

// 1. Limine requests start marker placed in `.limine_requests_start`
#[used]
#[link_section = ".limine_requests_start"]
pub static REQUESTS_START_MARKER: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

// 2. Base revision placed in `.limine_requests`
#[used]
#[link_section = ".limine_requests"]
pub static BASE_REVISION: LimineBaseRevision = LimineBaseRevision {
    id: [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc],
    revision: 2,
};

// 3. Higher-Half Direct Map request placed in `.limine_requests`
#[used]
#[link_section = ".limine_requests"]
pub static HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ],
    revision: 0,
    response: core::ptr::null(),
};

// 4. Memory Map request placed in `.limine_requests`
#[used]
#[link_section = ".limine_requests"]
pub static MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ],
    revision: 0,
    response: core::ptr::null(),
};

#[repr(C)]
pub struct LimineFramebuffer {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub unused: [u8; 7],
    pub edid_size: u64,
    pub edid: *const u8,
}

#[repr(C)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *const *const LimineFramebuffer,
}

#[repr(C)]
pub struct LimineFramebufferRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineFramebufferResponse,
}

unsafe impl Sync for LimineFramebufferRequest {}

// 5. Framebuffer request placed in `.limine_requests`
#[used]
#[link_section = ".limine_requests"]
pub static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: core::ptr::null(),
};

// 5. Limine requests end marker placed in `.limine_requests_end`
#[used]
#[link_section = ".limine_requests_end"]
pub static REQUESTS_END_MARKER: [u64; 2] = [
    0xadc0e0531bb10d03,
    0x9572709f31764c62,
];
