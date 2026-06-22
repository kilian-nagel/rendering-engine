use core::time;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::thread::sleep;

const DRM_IOCTL_BASE: u64 = 0x64;

const fn iowr(nr: u64, size: u64) -> u64 {
    // direction = read|write = 3, size in bits [29:16], type [15:8], nr [7:0]
    (3u64 << 30) | (size << 16) | (DRM_IOCTL_BASE << 8) | nr
}

const fn iow(nr: u64, size: u64) -> u64 {
    (1u64 << 30) | (size << 16) | (DRM_IOCTL_BASE << 8) | nr
}

const DRM_IOCTL_MODE_GETRESOURCES: u64 = iowr(0xA0, std::mem::size_of::<DrmModeCardRes>() as u64);
const DRM_IOCTL_MODE_GETCONNECTOR: u64 = iowr(0xA7, std::mem::size_of::<DrmModeGetConnector>() as u64);
const DRM_IOCTL_MODE_GETENCODER:   u64 = iowr(0xA6, std::mem::size_of::<DrmModeGetEncoder>() as u64);
const DRM_IOCTL_MODE_CREATE_DUMB:  u64 = iowr(0xB2, std::mem::size_of::<DrmModeCreateDumb>() as u64);
const DRM_IOCTL_MODE_MAP_DUMB:     u64 = iowr(0xB3, std::mem::size_of::<DrmModeMapDumb>() as u64);
const DRM_IOCTL_MODE_ADDFB:        u64 = iowr(0xAE, std::mem::size_of::<DrmModeFbCmd>() as u64);
const DRM_IOCTL_MODE_SETCRTC:      u64 = iowr(0xA2, std::mem::size_of::<DrmModeCrtc>() as u64);

#[repr(C)]
#[derive(Default)]
struct DrmModeCardRes {
    fb_id_ptr:        u64,
    crtc_id_ptr:      u64,
    connector_id_ptr: u64,
    encoder_id_ptr:   u64,
    count_fbs:        u32,
    count_crtcs:      u32,
    count_connectors: u32,
    count_encoders:   u32,
    min_width:        u32,
    max_width:        u32,
    min_height:       u32,
    max_height:       u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct DrmModeModeinfo {
    clock:       u32,
    hdisplay:    u16,
    hsync_start: u16,
    hsync_end:   u16,
    htotal:      u16,
    hskew:       u16,
    vdisplay:    u16,
    vsync_start: u16,
    vsync_end:   u16,
    vtotal:      u16,
    vscan:       u16,
    vrefresh:    u32,
    flags:       u32,
    kind:        u32,          
    name:        [u8; 32],
}

#[repr(C)]
#[derive(Default)]
struct DrmModeGetConnector {
    encoders_ptr:   u64,
    modes_ptr:      u64,
    props_ptr:      u64,
    prop_values_ptr:u64,
    count_modes:    u32,
    count_props:    u32,
    count_encoders: u32,
    encoder_id:     u32,   // current encoder
    connector_id:   u32,
    connector_type: u32,
    connector_type_id: u32,
    connection:     u32,   // 1 = connected
    mm_width:       u32,
    mm_height:      u32,
    subpixel:       u32,
    pad:            u32,
}

#[repr(C)]
#[derive(Default)]
struct DrmModeGetEncoder {
    encoder_id:   u32,
    encoder_type: u32,
    crtc_id:      u32,
    possible_crtcs:  u32,
    possible_clones: u32,
}

#[repr(C)]
#[derive(Default)]
struct DrmModeCreateDumb {
    height: u32,
    width:  u32,
    bpp:    u32,
    flags:  u32,
    handle: u32,
    pitch:  u32,
    size:   u64,
}

#[repr(C)]
#[derive(Default)]
struct DrmModeMapDumb {
    handle: u32,
    pad:    u32,
    offset: u64,
}

#[repr(C)]
#[derive(Default)]
struct DrmModeFbCmd {
    fb_id:  u32,
    width:  u32,
    height: u32,
    pitch:  u32,
    bpp:    u32,
    depth:  u32,
    handle: u32,
}

#[repr(C)]
#[derive(Default)]
struct DrmModeCrtc {
    set_connectors_ptr: u64,
    count_connectors:   u32,
    crtc_id:   u32,
    fb_id:     u32,
    x:         u32,
    y:         u32,
    gamma_size:u32,
    mode_valid:u32,
    mode:      DrmModeModeinfo,
}

// Ioctl wrapper
unsafe fn ioctl<T>(fd: i32, request: u64, arg: *mut T) -> i32 { unsafe {
    unsafe extern "C" {
        fn ioctl(fd: i32, request: u64, ...) -> i32;
    }
    ioctl(fd, request, arg)
}}


fn draw_frame(buffer: &mut [u32], height: u32, width: u32, pitch_pixels: usize, _frame: u32) {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let half: i32 = 100;

    // Fresh random color each frame so the 60 ms refresh is obvious.
    let r = rand::random::<u8>() as u32;
    let g = rand::random::<u8>() as u32;
    let b = rand::random::<u8>() as u32;
    let square_color = (r << 16) | (g << 8) | b;

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let inside = (x - cx).abs() < half && (y - cy).abs() < half;
            let color = if inside { square_color } else { 0x0000_0000 };
            buffer[y as usize * pitch_pixels + x as usize] = color;
        }
    }
}


// ──────────────────────────────────────────────────────────
//  Main
// ──────────────────────────────────────────────────────────
fn main() {
    // 1. Open the DRM device
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/dri/card1")
        .expect("Cannot open /dev/dri/card1 — are you in the 'video' group?");
    let fd = file.as_raw_fd();

    // 2. Get resource IDs (connectors, CRTCs …)
    let mut res = DrmModeCardRes::default();
    let mut connector_ids = vec![0u32; 8];
    let mut crtc_ids      = vec![0u32; 8];
    res.connector_id_ptr = connector_ids.as_mut_ptr() as u64;
    res.crtc_id_ptr      = crtc_ids.as_mut_ptr() as u64;
    res.count_connectors  = 8;
    res.count_crtcs       = 8;
    unsafe { ioctl(fd, DRM_IOCTL_MODE_GETRESOURCES, &mut res) };
    println!("connectors: {}  crtcs: {}", res.count_connectors, res.count_crtcs);

    // 3. Find the first connected connector
    let connector_id;
    let mode: DrmModeModeinfo;
    let encoder_id;
    loop {
        let mut found_connector = None;
        'outer: for i in 0..res.count_connectors as usize {
            let cid = connector_ids[i];

            let mut conn = DrmModeGetConnector { connector_id: cid, ..Default::default() };
            unsafe { ioctl(fd, DRM_IOCTL_MODE_GETCONNECTOR, &mut conn) };

            if conn.connection != 1 || conn.count_modes == 0 {
                continue;
            }
            let mut modes = vec![DrmModeModeinfo::default(); conn.count_modes as usize];
            conn.modes_ptr      = modes.as_mut_ptr() as u64;
            conn.count_modes    = modes.len() as u32;

            let mut encoders    = vec![0u32; conn.count_encoders as usize];
            conn.encoders_ptr   = encoders.as_mut_ptr() as u64;
            conn.count_encoders = encoders.len() as u32;
            unsafe { ioctl(fd, DRM_IOCTL_MODE_GETCONNECTOR, &mut conn) };

            found_connector = Some((cid, modes[0], conn.encoder_id));
            break 'outer;
        }
        let (cid, m, enc) = found_connector.expect("No connected display found");
        connector_id = cid;
        mode         = m;
        encoder_id   = enc;
        break;
    }

    let width  = mode.hdisplay as u32;
    let height = mode.vdisplay as u32;
    println!("Mode: {}x{}@{}Hz  connector={} encoder={}",
        width, height, mode.vrefresh, connector_id, encoder_id);

    // 4. Find the CRTC from the encoder
    let mut enc_info = DrmModeGetEncoder { encoder_id, ..Default::default() };
    unsafe { ioctl(fd, DRM_IOCTL_MODE_GETENCODER, &mut enc_info) };
    let crtc_id = enc_info.crtc_id;
    println!("Using CRTC id={}", crtc_id);

    // 5. Allocate a dumb framebuffer
    let mut create = DrmModeCreateDumb {
        width,
        height,
        bpp: 32,
        ..Default::default()
    };
    unsafe { ioctl(fd, DRM_IOCTL_MODE_CREATE_DUMB, &mut create) };
    println!("Dumb buffer: handle={} pitch={} size={}", create.handle, create.pitch, create.size);

    // 6. Register it as a KMS framebuffer
    let mut addfb = DrmModeFbCmd {
        width,
        height,
        bpp:    32,
        depth:  24,
        pitch:  create.pitch,
        handle: create.handle,
        ..Default::default()
    };
    unsafe { ioctl(fd, DRM_IOCTL_MODE_ADDFB, &mut addfb) };
    let fb_id = addfb.fb_id;
    println!("Framebuffer id={}", fb_id);

    // 7. Get the mmap offset for the dumb buffer
    let mut map_dumb = DrmModeMapDumb { handle: create.handle, ..Default::default() };
    unsafe { ioctl(fd, DRM_IOCTL_MODE_MAP_DUMB, &mut map_dumb) };

    // 8. mmap the framebuffer into our address space
    let fb_size = create.size as usize;
    let fb_ptr = unsafe {
        libc_mmap(
            std::ptr::null_mut(),
            fb_size,
            0x1 | 0x2,      // PROT_READ | PROT_WRITE
            0x01,            // MAP_SHARED
            fd,
            map_dumb.offset as i64,
        )
    };
    assert!(!fb_ptr.is_null() && fb_ptr as isize != -1, "mmap failed");
    let fb: &mut [u32] = unsafe {
        std::slice::from_raw_parts_mut(fb_ptr as *mut u32, fb_size / 4)
    };

    let pitch_pixels = create.pitch as usize / 4;


    // 10. Set the CRTC to the display
    let mut connector_id_copy = connector_id;
    let mut set_crtc = DrmModeCrtc {
        crtc_id,
        fb_id,
        x: 0,
        y: 0,
        set_connectors_ptr: &mut connector_id_copy as *mut u32 as u64,
        count_connectors: 1,
        mode_valid: 1,
        mode,
        ..Default::default()
    };

    println!("height : {}", height);
    println!("width : {}", width);


    let ret = unsafe { ioctl(fd, DRM_IOCTL_MODE_SETCRTC, &mut set_crtc) };
    if ret != 0 {
        eprintln!("SETCRTC failed (ret={}) — try running as root or with CAP_SYS_ADMIN", ret);
        return;
    }
    
    let mut frame: u32 = 0;
    loop {
        draw_frame(fb, height, width, pitch_pixels, frame);
        frame = frame.wrapping_add(1);
        sleep(time::Duration::from_secs(1));
    }
}

unsafe fn libc_mmap(
    addr:   *mut std::ffi::c_void,
    length: usize,
    prot:   i32,
    flags:  i32,
    fd:     i32,
    offset: i64,
) -> *mut std::ffi::c_void { unsafe {
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
}}
