mod drm;
mod rasterizer;

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;

use drm::create_framebuffer;
use drm::sys::{
    ioctl, DrmEventVblank, DrmModeCardRes, DrmModeCrtc, DrmModeCrtcPageFlip, DrmModeGetConnector,
    DrmModeGetEncoder, DrmModeModeinfo, DRM_IOCTL_MODE_GETCONNECTOR, DRM_IOCTL_MODE_GETENCODER,
    DRM_IOCTL_MODE_GETRESOURCES, DRM_IOCTL_MODE_PAGE_FLIP, DRM_IOCTL_MODE_SETCRTC,
    DRM_MODE_PAGE_FLIP_EVENT,
};
use rasterizer::{Color, Rasterizer, Rectangle, Triangle};

fn main() {
    // Open the DRM device
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/dri/card1")
        .expect("Cannot open /dev/dri/card1 — are you in the 'video' group?");
    let fd = file.as_raw_fd();

    // Get resource IDs (connectors, CRTCs …)
    let mut res = DrmModeCardRes::default();
    let mut connector_ids = vec![0u32; 8];
    let mut crtc_ids      = vec![0u32; 8];
    res.connector_id_ptr = connector_ids.as_mut_ptr() as u64;
    res.crtc_id_ptr      = crtc_ids.as_mut_ptr() as u64;
    res.count_connectors  = 8;
    res.count_crtcs       = 8;
    unsafe { ioctl(fd, DRM_IOCTL_MODE_GETRESOURCES, &mut res) };

    // Find the first connected connector
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

    // Find the CRTC from the encoder
    let mut enc_info = DrmModeGetEncoder { encoder_id, ..Default::default() };
    unsafe { ioctl(fd, DRM_IOCTL_MODE_GETENCODER, &mut enc_info) };
    let crtc_id = enc_info.crtc_id;

    // Allocate front and back buffer
    let mut bufs = [
        create_framebuffer(fd, width, height),
        create_framebuffer(fd, width, height),
    ];

    // Intialize CRTC by making it scan front buffer
    let mut connector_id_copy = connector_id;
    let mut set_crtc = DrmModeCrtc {
        crtc_id,
        fb_id: bufs[0].fb_id,
        x: 0,
        y: 0,
        set_connectors_ptr: &mut connector_id_copy as *mut u32 as u64,
        count_connectors: 1,
        mode_valid: 1,
        mode,
        ..Default::default()
    };

    let ret = unsafe { ioctl(fd, DRM_IOCTL_MODE_SETCRTC, &mut set_crtc) };
    if ret != 0 {
        eprintln!("SETCRTC failed (ret={}) — try running as root or with CAP_SYS_ADMIN", ret);
    }

    let mut log = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("/tmp/rendering.log")
        .expect("cannot open /tmp/rendering.log");

    // `front` is the buffer currently on screen, we draw into the other one (back)
    let mut front: usize = 0;
    let mut frame: u32 = 0;
    let mut offset_x: i32 = 0;
    let mut offset_y: i32 = 0;

    loop {
        let back = front ^ 1;

        // Draw next frame into the back buffer.
        let pitch_pixels = bufs[back].pitch_pixels;

        let rectangle = Rectangle {
            width: 200,
            height: 200,
            color: Color::Red,
        };
        if let Err(e) = rectangle.draw(bufs[back].pixels(), offset_x, offset_y, pitch_pixels) {
            writeln!(log, "rectangle draw failed: {}", e).ok();
        }

        let triangle = Triangle {
            p1: [100, 250],
            p2: [0, 450],
            p3: [200, 450],
            color: Color::Green,
        };
        if let Err(e) = triangle.draw(bufs[back].pixels(), offset_x, offset_y, pitch_pixels) {
            writeln!(log, "triangle draw failed: {}", e).ok();
        }

        // Schedule a page flip to the back buffer
        let mut flip = DrmModeCrtcPageFlip {
            crtc_id,
            fb_id: bufs[back].fb_id,
            flags: DRM_MODE_PAGE_FLIP_EVENT,
            ..Default::default()
        };
        let flip_ret = unsafe { ioctl(fd, DRM_IOCTL_MODE_PAGE_FLIP, &mut flip) };

        if flip_ret == 0 {
            // Wait for page flip to occur, it will occur once vblank event is written at next VSYNC
            let mut ev_buf = [0u8; std::mem::size_of::<DrmEventVblank>()];
            if let Err(e) = (&file).read_exact(&mut ev_buf) {
                writeln!(log, "reading flip event failed: {}", e).ok();
            }
            // The flip completed: the back buffer is now the front buffer.
            front = back;
        } else {
            // Page flip failed we reassert the CRTC to still display the framebuffer
            set_crtc.fb_id = bufs[back].fb_id;
            let setcrtc_ret = unsafe { ioctl(fd, DRM_IOCTL_MODE_SETCRTC, &mut set_crtc) };
            writeln!(log, "page flip failed (ret={}), setcrtc={}", flip_ret, setcrtc_ret).ok();
            front = back;
        }

        writeln!(log, "presented frame {} on buffer {} (flip={})", frame, back, flip_ret).ok();
        log.flush().ok();
        frame = frame.wrapping_add(1);
    }
}
