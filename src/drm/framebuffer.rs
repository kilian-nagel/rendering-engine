//! One scanout buffer: a dumb GPU buffer registered as a KMS framebuffer and
//! mmap'd into our address space so we can draw into it from the CPU.

use super::sys::{
    self, DrmModeCreateDumb, DrmModeFbCmd, DrmModeMapDumb, DRM_IOCTL_MODE_ADDFB,
    DRM_IOCTL_MODE_CREATE_DUMB, DRM_IOCTL_MODE_MAP_DUMB,
};

pub struct Framebuffer {
    pub fb_id:        u32,
    ptr:              *mut u32,
    len:              usize,
    pub pitch_pixels: usize,
}

impl Framebuffer {
    pub fn pixels(&mut self) -> &mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

/// Allocate a dumb buffer, register it as a KMS framebuffer and mmap it.
pub fn create_framebuffer(fd: i32, width: u32, height: u32) -> Framebuffer {
    // Allocate the dumb buffer.
    let mut create = DrmModeCreateDumb { width, height, bpp: 32, ..Default::default() };
    unsafe { sys::ioctl(fd, DRM_IOCTL_MODE_CREATE_DUMB, &mut create) };

    // Register it as a KMS framebuffer.
    let mut addfb = DrmModeFbCmd {
        width,
        height,
        bpp:    32,
        depth:  24,
        pitch:  create.pitch,
        handle: create.handle,
        ..Default::default()
    };
    unsafe { sys::ioctl(fd, DRM_IOCTL_MODE_ADDFB, &mut addfb) };

    // Get the mmap offset and map it into our address space.
    let mut map_dumb = DrmModeMapDumb { handle: create.handle, ..Default::default() };
    unsafe { sys::ioctl(fd, DRM_IOCTL_MODE_MAP_DUMB, &mut map_dumb) };

    let fb_size = create.size as usize;
    let fb_ptr = unsafe {
        sys::libc_mmap(
            std::ptr::null_mut(),
            fb_size,
            0x1 | 0x2, // PROT_READ | PROT_WRITE
            0x01,      // MAP_SHARED
            fd,
            map_dumb.offset as i64,
        )
    };
    assert!(!fb_ptr.is_null() && fb_ptr as isize != -1, "mmap failed");

    println!(
        "Framebuffer id={} handle={} pitch={} size={}",
        addfb.fb_id, create.handle, create.pitch, create.size
    );

    Framebuffer {
        fb_id:        addfb.fb_id,
        ptr:          fb_ptr as *mut u32,
        len:          fb_size / 4,
        pitch_pixels: create.pitch as usize / 4,
    }
}
