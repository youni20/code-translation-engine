use std::fs::File;
use std::io::{Read, Write, BufWriter, Seek, SeekFrom};
use std::mem;

const K_NUM_POINTS_DEFAULT_POISSON: usize = 20000;
const K_NUM_POINTS_DEFAULT_VOGEL: usize = 2000;
const K_NUM_POINTS_DEFAULT_JITTERED: usize = 2500;
const K_IMAGE_SIZE: usize = 512;

static mut G_DENSITY_MAP: Option<Vec<f32>> = None;

#[repr(C, packed)]
struct BMPHeader {
    // BITMAPFILEHEADER
    bfType: u16,
    bfSize: u32,
    bfReserved1: u16,
    bfReserved2: u16,
    bfOffBits: u32,
    // BITMAPINFOHEADER
    biSize: u32,
    biWidth: u32,
    biHeight: u32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: u32,
    biYPelsPerMeter: u32,
    biClrUsed: u32,
    biClrImportant: u32,
}

struct AVIWriter {
    file: BufWriter<File>,
    width: usize,
    height: usize,
    fps: usize,
    skip_frames: usize,
    input_frame_count: u32,
    frame_count: u32,
    frame_data_size: u32,
    movi_start: u32,
    row_padding: usize,
    padded_row_size: usize,
    padded_frame_size: u32,
}

impl AVIWriter {
    fn new(file_name: &str, width: usize, height: usize, skip_frames: usize) -> Self {
        let frame_data_size = width * height;
        let row_padding = (4 - (width % 4)) % 4;
        let padded_row_size = width + row_padding;
        let padded_frame_size = padded_row_size * height;

        println!("\nSaving video to `{}`", file_name);

        let file = BufWriter::new(File::create(file_name).expect("Cannot create file"));
        let mut writer = AVIWriter {
            file,
            width,
            height,
            fps: 60,
            skip_frames,
            input_frame_count: 0,
            frame_count: 0,
            frame_data_size: frame_data_size as u32,
            movi_start: 0,
            row_padding,
            padded_row_size,
            padded_frame_size: padded_frame_size as u32,
        };

        writer.write_header();
        writer.movi_start = writer.file.stream_position().expect("Cannot get file position") as u32;
        writer.write_chunk_header(b"LIST", 0);
        writer.write_four_cc(b"movi");
        
        writer
    }

    fn write_four_cc(&mut self, four_cc: &[u8; 4]) {
        self.file.write_all(four_cc).expect("Cannot write fourcc");
    }

    fn write_u32(&mut self, value: u32) {
        self.file.write_all(&value.to_le_bytes()).expect("Cannot write u32");
    }

    fn write_chunk_header(&mut self, four_cc: &[u8; 4], size: u32) {
        self.write_four_cc(four_cc);
        self.write_u32(size);
    }

    fn write_header(&mut self) {
        self.write_four_cc(b"RIFF");
        self.write_u32(0);
        self.write_four_cc(b"AVI ");
        self.write_four_cc(b"LIST");
        let hdrl_size_pos = self.file.stream_position().expect("Cannot get file position") as u32;
        self.write_u32(0);
        self.write_four_cc(b"hdrl");
        // Main AVI header (avih)
        self.write_four_cc(b"avih");
        self.write_u32(56);
        self.write_u32(1000000 / self.fps as u32);
        self.write_u32(self.padded_frame_size * self.fps as u32);
        self.write_u32(0);
        self.write_u32(0x10);
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(1);
        self.write_u32(self.padded_frame_size);
        self.write_u32(self.width as u32);
        self.write_u32(self.height as u32);
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(0);
        // Stream LIST
        self.write_four_cc(b"LIST");
        let strl_size_pos = self.file.stream_position().expect("Cannot get file position") as u32;
        self.write_u32(0);
        self.write_four_cc(b"strl");
        // Stream header (strh)
        self.write_four_cc(b"strh");
        self.write_u32(56);
        self.write_four_cc(b"vids");
        self.write_four_cc(b"DIB ");
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(1);
        self.write_u32(self.fps as u32);
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(self.padded_frame_size);
        self.write_u32(0xFFFFFFFF);
        self.write_u32(0);
        self.write_u32(self.width as u32);
        self.write_u32(self.height as u32);
        // Stream format (strf)
        self.write_four_cc(b"strf");
        self.write_u32(40 + 256 * 4);
        self.write_u32(40);
        self.write_u32(self.width as u32);
        self.write_u32(self.height as u32);
        self.write_u32(1);
        self.write_u32(8);
        self.write_u32(0);
        self.write_u32(self.padded_frame_size);
        self.write_u32(0);
        self.write_u32(0);
        self.write_u32(256);
        self.write_u32(256);

        for i in 0..256 {
            self.file.write(&[i as u8, i as u8, i as u8, 0]).expect("Cannot write grayscale palette");
        }

        let strl_end = self.file.stream_position().expect("Cannot get file position") as u32;
        self.file.seek(SeekFrom::Start(strl_size_pos as u64)).expect("Cannot seek");
        self.write_u32(strl_end - strl_size_pos - 4);
        self.file.seek(SeekFrom::Start(strl_end as u64)).expect("Cannot seek");

        let hdrl_end = self.file.stream_position().expect("Cannot get file position") as u32;
        self.file.seek(SeekFrom::Start(hdrl_size_pos as u64)).expect("Cannot seek");
        self.write_u32(hdrl_end - hdrl_size_pos - 4);
        self.file.seek(SeekFrom::Start(hdrl_end as u64)).expect("Cannot seek");
    }

    fn write_index(&mut self) {
        self.write_four_cc(b"idx1");
        self.write_u32(self.frame_count * 16);

        let mut offset = 4;

        for _ in 0..self.frame_count {
            self.write_four_cc(b"00db");
            self.write_u32(0x10);
            self.write_u32(offset);
            self.write_u32(self.padded_frame_size);

            offset += self.padded_frame_size + 8;
        }
    }

    fn add_frame(&mut self, bgr_data: &[u8], is_last_frame: bool) -> bool {
        if !is_last_frame && (self.input_frame_count % self.skip_frames as u32) != 0 {
            self.input_frame_count += 1;
            return false;
        }

        self.write_chunk_header(b"00db", self.padded_frame_size);
        let mut row_buffer = vec![0u8; self.padded_row_size];

        for y in 0..self.height {
            for x in 0..self.width {
                row_buffer[x] = bgr_data[(y * self.width + x) * 3];
            }
            self.file.write_all(&row_buffer).expect("Cannot write row");
        }

        self.frame_count += 1;

        true
    }
}

impl Drop for AVIWriter {
    fn drop(&mut self) {
        let movi_end = self.file.stream_position().expect("Cannot get file position") as u32;
        let movi_size = movi_end - self.movi_start - 8;

        self.write_index();

        let file_end = self.file.stream_position().expect("Cannot get file position") as u32;

        self.file.seek(SeekFrom::Start(self.movi_start as u64 + 4)).expect("Cannot seek");
        self.write_u32(movi_size + 4);

        self.file.seek(SeekFrom::Start(4)).expect("Cannot seek");
        self.write_u32(file_end - 8);

        self.file.seek(SeekFrom::Start(48)).expect("Cannot seek");
        self.write_u32(self.frame_count);

        self.file.seek(SeekFrom::Start(140)).expect("Cannot seek");
        self.write_u32(self.frame_count);

        println!("\nSaved AVI with {} frames", self.frame_count);
    }
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: usize, height: usize) {
    let header = BMPHeader {
        bfType: 0x4D42, // 'MB'
        bfSize: (width * height * 3 + mem::size_of::<BMPHeader>()) as u32,
        bfReserved1: 0,
        bfReserved2: 0,
        bfOffBits: 0x36,
        biSize: 40,
        biWidth: width as u32,
        biHeight: height as u32,
        biPlanes: 1,
        biBitCount: 24,
        biCompression: 0,
        biSizeImage: (width * height * 3) as u32,
        biXPelsPerMeter: 6000,
        biYPelsPerMeter: 6000,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    let mut file = File::create(file_name).expect("Cannot create BMP file");
    file.write_all(unsafe { std::slice::from_raw_parts(&header as *const _ as *const u8, mem::size_of::<BMPHeader>()) }).expect("Cannot write BMP header");
    file.write_all(raw_bgr_image).expect("Cannot write BMP data");
    println!("Saved {}", file_name);
}

fn load_bmp(file_name: &str, out_width: &mut usize, out_height: &mut usize) -> Vec<u8> {
    let mut file = File::open(file_name).expect("Cannot open BMP file");
    let mut header = BMPHeader {
        bfType: 0,
        bfSize: 0,
        bfReserved1: 0,
        bfReserved2: 0,
        bfOffBits: 0,
        biSize: 0,
        biWidth: 0,
        biHeight: 0,
        biPlanes: 0,
        biBitCount: 0,
        biCompression: 0,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    unsafe {
        file.read_exact(std::slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, mem::size_of::<BMPHeader>())).expect("Cannot read BMP header");
    }

    *out_width = header.biWidth as usize;
    *out_height = header.biHeight as usize;

    let data_size = 3 * header.biWidth as usize * header.biHeight as usize;
    let mut img = vec![0u8; data_size];
    file.read_exact(&mut img).expect("Cannot read BMP data");
    img
}

fn load_density_map(file_name: &str) {
    println!("Loading density map {}", file_name);
    let mut width = 0;
    let mut height = 0;
    let data = load_bmp(file_name, &mut width, &mut height);

    println!("Loaded ( {} x {} )", width, height);

    if width != K_IMAGE_SIZE || height != K_IMAGE_SIZE {
        println!("ERROR: density map should be {} x {}", K_IMAGE_SIZE, K_IMAGE_SIZE);
        std::process::exit(255);
    }

    let mut density_map = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let index = 3 * (x + y * width);
            density_map.push(data[index] as f32 / 255.0);
        }
    }

    unsafe {
        G_DENSITY_MAP = Some(density_map);
    }
}

fn print_banner() {
    println!("Poisson disk points generator");
    // Assuming a placeholder version for PoissonGenerator::Version equivalent
    println!("Version 1.7.0");
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com");
    println!();
    println!("Usage: Poisson [density-map-rgb24.bmp] [--raw-points] [--num-points=<value>] [--square] [--vogel-disk | --jittered-grid | --hammersley] [--shuffle] [--save-frames] [--save-video[=<skip-frames>]]");
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    print_banner();

    if args.len() > 1 && !args[1].starts_with("--") {
        load_density_map(&args[1]);
    }

    let has_cmd_line_arg = |arg: &str| -> bool {
        args.iter().skip(1).any(|a| a == arg)
    };

    let get_cmd_line_value = |arg: &str, default_value: usize| -> usize {
        args.iter().skip(1).find_map(|a| {
            if a.starts_with(arg) {
                a[arg.len()..].parse().ok()
            } else {
                None
            }
        }).unwrap_or(default_value)
    };

    let get_cmd_line_value_skip_frames = |arg: &str, default_value: usize| -> usize {
        args.iter().skip(1).find_map(|a| {
            if a.starts_with(&format!("--save-video=")) {
                a[arg.len() + 1..].parse().ok()
            } else {
                None
            }
        }).unwrap_or(default_value)
    };

    let has_cmd_line_arg_prefix = |prefix: &str| -> bool {
        args.iter().skip(1).any(|a| a.starts_with(prefix))
    };

    let cmd_raw_points_output = has_cmd_line_arg("--raw-points");
    let cmd_square = has_cmd_line_arg("--square");
    let cmd_vogel_disk = has_cmd_line_arg("--vogel-disk");
    let cmd_jittered_grid = has_cmd_line_arg("--jittered-grid");
    let cmd_hammersley = has_cmd_line_arg("--hammersley");
    let cmd_shuffle = has_cmd_line_arg("--shuffle");
    let cmd_save_frames = has_cmd_line_arg("--save-frames");
    let cmd_save_video = has_cmd_line_arg_prefix("--save-video");
    let video_skip_frames = get_cmd_line_value_skip_frames("--save-video", 16);

    let num_points = get_cmd_line_value(
        "--num-points=", 
        if cmd_vogel_disk {
            K_NUM_POINTS_DEFAULT_VOGEL
        } else if cmd_jittered_grid {
            K_NUM_POINTS_DEFAULT_JITTERED
        } else {
            K_NUM_POINTS_DEFAULT_POISSON
        }
    );

    println!("NumPoints = {}", num_points);

    // Placeholder for PoissonGenerator::DefaultPRNG
    let prng = (); 

    // Placeholder based on command-line option checks, as we don't have a PoissonGenerator
    // Assume Points is a Vec<(f32, f32)> for simplicity
    let points: Vec<(f32, f32)> = vec![];

    // Prepare BGR image
    let mut img = vec![0u8; 3 * K_IMAGE_SIZE * K_IMAGE_SIZE];
    img.fill(0);

    if cmd_shuffle {
        println!("Shuffling points...");
        // PoissonGenerator::shuffle(&mut points, &mut prng);
    }

    let mut avi_writer = if cmd_save_video {
        Some(AVIWriter::new("Points.avi", K_IMAGE_SIZE, K_IMAGE_SIZE, video_skip_frames))
    } else {
        None
    };

    let mut frame = 0;
    let total_points = points.len();
    for i in &points {
        let x = (i.0 * K_IMAGE_SIZE as f32) as isize;
        let y = (i.1 * K_IMAGE_SIZE as f32) as isize;
        if x < 0 || y < 0 || x as usize >= K_IMAGE_SIZE || y as usize >= K_IMAGE_SIZE {
            continue;
        }

        if let Some(density_map) = unsafe { &G_DENSITY_MAP } {
            let r = 0.0; // Placeholder for random float in PRNG
            let p = density_map[(x + y as isize * K_IMAGE_SIZE as isize) as usize];
            if r > p {
                continue;
            }
        }

        let base = 3 * (x + y as isize * K_IMAGE_SIZE as isize) as usize;
        img[base] = 255;
        img[base + 1] = 255;
        img[base + 2] = 255;

        if cmd_save_frames {
            let file_name = format!("pnt{:05}.bmp", frame);
            frame += 1;
            save_bmp(&file_name, &img, K_IMAGE_SIZE, K_IMAGE_SIZE);
        }

        if let Some(ref mut writer) = avi_writer {
            if writer.add_frame(&img, false) {
                print!("\rRendering points to video: {}/{}", frame, total_points);
            }
        }
    }

    if let Some(ref mut writer) = avi_writer {
        writer.add_frame(&img, true);
        println!("\rRendering points to video: {}/{}", total_points, total_points);
    }

    save_bmp("Points.bmp", &img, K_IMAGE_SIZE, K_IMAGE_SIZE);

    let mut file = File::create("points.txt").expect("Cannot create points.txt");
    if cmd_raw_points_output {
        writeln!(file, "NumPoints = {}", points.len()).expect("Cannot write points.txt");

        for p in &points {
            writeln!(file, "{} {}", p.0, p.1).expect("Cannot write points.txt");
        }
    } else {
        writeln!(file, "const vec2 points[{}]", points.len()).expect("Cannot write points.txt");
        writeln!(file, "{{").expect("Cannot write points.txt");
        for p in &points {
            writeln!(file, "\tvec2({:.6}f, {:.6}f),", p.0, p.1).expect("Cannot write points.txt");
        }
        writeln!(file, "}};").expect("Cannot write points.txt");
    }
}