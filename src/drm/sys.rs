//! Raw DRM/KMS ioctl bindings: constants, wire structs and the ioctl syscall wrapper.

const DRM_IOCTL_BASE: u64 = 0x64;

const fn iowr(nr: u64, size: u64) -> u64 {
    // direction = read|write = 3, size in bits [29:16], type [15:8], nr [7:0]
    (3u64 << 30) | (size << 16) | (DRM_IOCTL_BASE << 8) | nr
}

pub const DRM_IOCTL_MODE_GETRESOURCES: u64 = iowr(0xA0, std::mem::size_of::<DrmModeCardRes>() as u64);
pub const DRM_IOCTL_MODE_GETCONNECTOR: u64 = iowr(0xA7, std::mem::size_of::<DrmModeGetConnector>() as u64);
pub const DRM_IOCTL_MODE_GETENCODER:   u64 = iowr(0xA6, std::mem::size_of::<DrmModeGetEncoder>() as u64);
pub const DRM_IOCTL_MODE_CREATE_DUMB:  u64 = iowr(0xB2, std::mem::size_of::<DrmModeCreateDumb>() as u64);
pub const DRM_IOCTL_MODE_MAP_DUMB:     u64 = iowr(0xB3, std::mem::size_of::<DrmModeMapDumb>() as u64);
pub const DRM_IOCTL_MODE_ADDFB:        u64 = iowr(0xAE, std::mem::size_of::<DrmModeFbCmd>() as u64);
pub const DRM_IOCTL_MODE_SETCRTC:      u64 = iowr(0xA2, std::mem::size_of::<DrmModeCrtc>() as u64);
pub const DRM_IOCTL_MODE_PAGE_FLIP:    u64 = iowr(0xB0, std::mem::size_of::<DrmModeCrtcPageFlip>() as u64);

pub const DRM_MODE_PAGE_FLIP_EVENT: u32 = 0x01;

#[repr(C)]
#[derive(Default)]
pub struct DrmModeCardRes {
    pub fb_id_ptr:        u64,
    pub crtc_id_ptr:      u64,
    pub connector_id_ptr: u64,
    pub encoder_id_ptr:   u64,
    pub count_fbs:        u32,
    pub count_crtcs:      u32,
    pub count_connectors: u32,
    pub count_encoders:   u32,
    pub min_width:        u32,
    pub max_width:        u32,
    pub min_height:       u32,
    pub max_height:       u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct DrmModeModeinfo {
    pub clock:       u32,
    pub hdisplay:    u16,
    pub hsync_start: u16,
    pub hsync_end:   u16,
    pub htotal:      u16,
    pub hskew:       u16,
    pub vdisplay:    u16,
    pub vsync_start: u16,
    pub vsync_end:   u16,
    pub vtotal:      u16,
    pub vscan:       u16,
    pub vrefresh:    u32,
    pub flags:       u32,
    pub kind:        u32,
    pub name:        [u8; 32],
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeGetConnector {
    pub encoders_ptr:    u64,
    pub modes_ptr:       u64,
    pub props_ptr:       u64,
    pub prop_values_ptr: u64,
    pub count_modes:     u32,
    pub count_props:     u32,
    pub count_encoders:  u32,
    pub encoder_id:      u32, // current encoder
    pub connector_id:    u32,
    pub connector_type:  u32,
    pub connector_type_id: u32,
    pub connection:      u32, // 1 = connected
    pub mm_width:         u32,
    pub mm_height:        u32,
    pub subpixel:         u32,
    pub pad:              u32,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeGetEncoder {
    pub encoder_id:      u32,
    pub encoder_type:    u32,
    pub crtc_id:         u32,
    pub possible_crtcs:  u32,
    pub possible_clones: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeCreateDumb {
    pub height: u32,
    pub width:  u32,
    pub bpp:    u32,
    pub flags:  u32,
    pub handle: u32,
    pub pitch:  u32,
    pub size:   u64,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeMapDumb {
    pub handle: u32,
    pub pad:    u32,
    pub offset: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeFbCmd {
    pub fb_id:  u32,
    pub width:  u32,
    pub height: u32,
    pub pitch:  u32,
    pub bpp:    u32,
    pub depth:  u32,
    pub handle: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeCrtcPageFlip {
    pub crtc_id:   u32,
    pub fb_id:     u32,
    pub flags:     u32,
    pub reserved:  u32,
    pub user_data: u64,
}

#[repr(C)]
pub struct DrmEventVblank {
    pub kind:      u32,
    pub length:    u32,
    pub user_data: u64,
    pub tv_sec:    u32,
    pub tv_usec:   u32,
    pub sequence:  u32,
    pub crtc_id:   u32,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeCrtc {
    pub set_connectors_ptr: u64,
    pub count_connectors:   u32,
    pub crtc_id:    u32,
    pub fb_id:      u32,
    pub x:          u32,
    pub y:          u32,
    pub gamma_size: u32,
    pub mode_valid: u32,
    pub mode:       DrmModeModeinfo,
}

/// Thin wrapper around the variadic libc `ioctl`.
pub unsafe fn ioctl<T>(fd: i32, request: u64, arg: *mut T) -> i32 {
    unsafe {
        unsafe extern "C" {
            fn ioctl(fd: i32, request: u64, ...) -> i32;
        }
        ioctl(fd, request, arg)
    }
}

pub unsafe fn libc_mmap(
    addr:   *mut std::ffi::c_void,
    length: usize,
    prot:   i32,
    flags:  i32,
    fd:     i32,
    offset: i64,
) -> *mut std::ffi::c_void {
    unsafe {
        unsafe extern "C" {
            fn mmap(
                addr:   *mut std::ffi::c_void,
                length: usize,
                prot:   i32,
                flags:  i32,
                fd:     i32,
                offset: i64,
            ) -> *mut std::ffi::c_void;
        }
        mmap(addr, length, prot, flags, fd, offset)
    }
}
