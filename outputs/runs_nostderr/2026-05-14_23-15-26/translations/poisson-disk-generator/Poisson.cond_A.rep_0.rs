use std::fs::File;
use std::io::{self, Write, Read};
use std::mem::size_of;
use std::ffi::CStr;
use std::ffi::CString;
use std::env;
use std::ptr;
use std::slice;

const K_NUM_POINTS_DEFAULT_POISSON: usize = 20000;
const K_NUM_POINTS_DEFAULT_VOGEL: usize = 2000;
const K_NUM_POINTS_DEFAULT_JITTERED: usize = 2500;
const K_IMAGE_SIZE: usize = 512;

struct SBmpHeader {
    bf_type: u16,
    bf_size: u32,
    bf_reserved1: u16,
    bf_reserved2: u16,
    bf_off_bits: u32,
    bi_size: u32,
    bi_width: u32,
    bi_height: u32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_xpels_per_meter: u32,
    bi_ypels_per_meter: u32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

impl SBmpHeader {
    fn new() -> Self {
        SBmpHeader {
            bf_type: 0x4D * 256 + 0x42,
            bf_size: 0,
            bf_reserved1: 0,
            bf_reserved2: 0,
            bf_off_bits: 0x36,
            bi_size: 40,
            bi_width: 0,
            bi_height: 0,
            bi_planes: 1,
            bi_bit_count: 24,
            bi_compression: 0,
            bi_size_image: 0,
            bi_xpels_per_meter: 6000,
            bi_ypels_per_meter: 6000,
            bi_clr_used: 0,
            bi_clr_important: 0,
        }
    }
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: usize, height: usize) -> io::Result<()> {
    let mut header = SBmpHeader::new();
    let image_size = width * height * 3;
    header.bf_size = image_size as u32 + size_of::<SBmpHeader>() as u32;
    header.bi_width = width as u32;
    header.bi_height = height as u32;
    header.bi_size_image = image_size as u32;

    let mut file = File::create(file_name)?;
    file.write_all(unsafe { slice::from_raw_parts(&header as *const _ as *const u8, size_of::<SBmpHeader>()) })?;
    file.write_all(raw_bgr_image)?;
    println!("Saved {}", file_name);
    Ok(())
}

fn load_bmp(file_name: &str, out_width: &mut i32, out_height: &mut i32) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_name)?;
    let mut header = SBmpHeader::new();
    file.read_exact(unsafe { slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, size_of::<SBmpHeader>()) })?;

    *out_width = header.bi_width as i32;
    *out_height = header.bi_height as i32;

    let data_size = 3 * header.bi_width as usize * header.bi_height as usize;
    let mut img = vec![0u8; data_size];
    file.read_exact(&mut img)?;

    Ok(img)
}

fn print_banner() {
    println!("Poisson disk points generator");
    println!("Version 1.7.0");
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com");
    println!();
    println!("Usage: Poisson [density-map-rgb24.bmp] [--raw-points] [--num-points=<value>] [--square] [--vogel-disk | --jittered-grid | --hammersley] [--shuffle] [--save-frames] [--save-video[=<skip-frames>]]");
    println!();
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && !args[1].starts_with("--") {
        load_density_map(&args[1]).expect("Failed to load density map");
    }

    // Placeholder for the actual application logic.
    // This part would involve a conversion of the rest of the C++ logic to Rust,
    // including the use of a Poisson disk generator library or custom implementation.
}

fn load_density_map(file_name: &str) -> io::Result<()> {
    println!("Loading density map {}", file_name);

    let mut width = 0;
    let mut height = 0;
    let data = load_bmp(file_name, &mut width, &mut height)?;

    println!("Loaded ( {} x {} )", width, height);

    if width != K_IMAGE_SIZE as i32 || height != K_IMAGE_SIZE as i32 {
        println!("ERROR: density map should be {} x {}", K_IMAGE_SIZE, K_IMAGE_SIZE);
        std::process::exit(255);
    }

    unsafe {
        let mut g_density_map: Vec<f32> = vec![0.0; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                g_density_map[(x + y * width) as usize] =
                    data[(3 * (x + y * width)) as usize] as f32 / 255.0;
            }
        }
    }

    Ok(())
}