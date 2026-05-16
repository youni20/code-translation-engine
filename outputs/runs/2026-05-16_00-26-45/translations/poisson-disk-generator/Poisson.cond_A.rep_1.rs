use std::fs::File;
use std::io::{self, Read, Write};

const K_NUM_POINTS_DEFAULT_POISSON: u32 = 20000;
const K_NUM_POINTS_DEFAULT_VOGEL: u32 = 2000;
const K_NUM_POINTS_DEFAULT_JITTERED: u32 = 2500;
const K_IMAGE_SIZE: usize = 512;

static mut G_DENSITY_MAP: Option<Vec<f32>> = None;

#[repr(C, packed)]
struct BMPHeader {
    bf_type: u16,
    bf_size: u32,
    bf_reserved1: u16,
    bf_reserved2: u16,
    bf_off_bits: u32,
    bi_size: u32,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_x_pels_per_meter: i32,
    bi_y_pels_per_meter: i32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

struct AVIWriter {
    file: File,
    width: usize,
    height: usize,
    fps: u32,
    skip_frames: u32,
    input_frame_count: u32,
    frame_count: u32,
    frame_data_size: u32,
    movi_start: u32,
    row_padding: usize,
    padded_row_size: usize,
    padded_frame_size: u32,
}

impl AVIWriter {
    pub fn new(file_name: &str, width: usize, height: usize, skip_frames: u32) -> io::Result<Self> {
        let frame_data_size = (width as u32 * height as u32) as u32;
        let row_padding = (4 - (width as u32) % 4) % 4;
        let padded_row_size = width + row_padding as usize;
        let padded_frame_size = (padded_row_size * height) as u32;

        println!("\nSaving video to `{}`", file_name);

        let file = File::create(file_name)?;

        let mut writer = AVIWriter {
            file,
            width,
            height,
            fps: 60,
            skip_frames,
            input_frame_count: 0,
            frame_count: 0,
            frame_data_size,
            movi_start: 0,
            row_padding: row_padding as usize,
            padded_row_size,
            padded_frame_size,
        };

        writer.write_header()?;
        writer.movi_start = writer.file.metadata()?.len() as u32;

        writer.write_chunk_header("LIST", 0)?;
        writer.write_four_cc("movi")?;

        Ok(writer)
    }

    pub fn add_frame(&mut self, bgr_data: &[u8], is_last_frame: bool) -> io::Result<bool> {
        if !is_last_frame && (self.input_frame_count % self.skip_frames) != 0 {
            self.input_frame_count += 1;
            return Ok(false);
        }

        self.write_chunk_header("00db", self.padded_frame_size)?;

        let mut row_buffer = vec![0u8; self.padded_row_size];
        for y in 0..self.height {
            for x in 0..self.width {
                row_buffer[x] = bgr_data[(y * self.width + x) * 3];
            }
            self.file.write_all(&row_buffer)?;
        }

        self.frame_count += 1;

        Ok(true)
    }

    fn write_four_cc(&mut self, fourcc: &str) -> io::Result<()> {
        self.file.write_all(fourcc.as_bytes())?;
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.file.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_u16(&mut self, value: u16) -> io::Result<()> {
        self.file.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_chunk_header(&mut self, fourcc: &str, size: u32) -> io::Result<()> {
        self.write_four_cc(fourcc)?;
        self.write_u32(size)?;
        Ok(())
    }

    fn write_header(&mut self) -> io::Result<()> {
        self.write_four_cc("RIFF")?;
        self.write_u32(0)?;
        self.write_four_cc("AVI ")?;
        self.write_four_cc("LIST")?;
        let _hdrl_size_pos = self.file.metadata()?.len() as u32;
        self.write_u32(0)?;
        self.write_four_cc("hdrl")?;
        self.write_four_cc("avih")?;
        self.write_u32(56)?;
        self.write_u32(1_000_000 / self.fps)?;
        self.write_u32(self.padded_frame_size * self.fps)?;
        self.write_u32(0)?;
        self.write_u32(0x10)?;
        self.write_u32(0)?;
        self.write_u32(0)?;
        self.write_u32(1)?;
        self.write_u32(self.padded_frame_size)?;
        self.write_u32(self.width as u32)?;
        self.write_u32(self.height as u32)?;
        for _ in 0..4 {
            self.write_u32(0)?;
        }

        self.write_four_cc("LIST")?;
        let _strl_size_pos = self.file.metadata()?.len() as u32;
        self.write_u32(0)?;
        self.write_four_cc("strl")?;
        self.write_four_cc("strh")?;
        self.write_u32(56)?;
        self.write_four_cc("vids")?;
        self.write_four_cc("DIB ")?;
        self.write_u32(0)?;
        self.write_u16(0)?;
        self.write_u16(0)?;
        self.write_u32(0)?;
        self.write_u32(1)?;
        self.write_u32(self.fps)?;
        self.write_u32(0)?;
        self.write_u32(0)?;
        self.write_u32(self.padded_frame_size)?;
        self.write_u32(0xFFFFFFFF)?;
        self.write_u32(0)?;
        self.write_u16(0)?;
        self.write_u16(0)?;
        self.write_u16(self.width as u16)?;
        self.write_u16(self.height as u16)?;

        self.write_four_cc("strf")?;
        self.write_u32(40 + 256 * 4)?;
        self.write_u32(40)?;
        self.write_u32(self.width as u32)?;
        self.write_u32(self.height as u32)?;
        self.write_u16(1)?;
        self.write_u16(8)?;
        self.write_u32(0)?;
        self.write_u32(self.padded_frame_size)?;
        self.write_u32(0)?;
        self.write_u32(0)?;
        self.write_u32(256)?;
        self.write_u32(256)?;

        for i in 0..256 {
            self.file.write_all(&[i as u8, i as u8, i as u8, 0])?;
        }

        let strl_end = self.file.metadata()?.len() as u64;
        self.file.set_len(strl_end)?;

        let hdrl_end = self.file.metadata()?.len() as u64;
        self.file.set_len(hdrl_end)?;

        Ok(())
    }

    fn write_index(&mut self) -> io::Result<()> {
        self.write_four_cc("idx1")?;
        self.write_u32(self.frame_count * 16)?;

        let mut offset = 4;

        for _ in 0..self.frame_count {
            self.write_four_cc("00db")?;
            self.write_u32(0x10)?;
            self.write_u32(offset)?;
            self.write_u32(self.padded_frame_size)?;

            offset += self.padded_frame_size + 8;
        }

        Ok(())
    }
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: i32, height: i32) -> io::Result<()> {
    let header = BMPHeader {
        bf_type: 0x4D * 256 + 0x42,
        bf_size: (width * height * 3) as u32 + std::mem::size_of::<BMPHeader>() as u32,
        bf_reserved1: 0,
        bf_reserved2: 0,
        bf_off_bits: 0x36,
        bi_size: 40,
        bi_width: width,
        bi_height: height,
        bi_planes: 1,
        bi_bit_count: 24,
        bi_compression: 0,
        bi_size_image: (width * height * 3) as u32,
        bi_x_pels_per_meter: 6000,
        bi_y_pels_per_meter: 6000,
        bi_clr_used: 0,
        bi_clr_important: 0,
    };

    let mut file = File::create(file_name)?;

    write!(file, "{:?}", &header as *const _ as *const [u8; 2])?;
    file.write_all(raw_bgr_image)?;

    println!("Saved {}", file_name);
    Ok(())
}

fn load_bmp(file_name: &str) -> io::Result<(Vec<u8>, i32, i32)> {
    let mut file = File::open(file_name)?;
    let mut header = BMPHeader {
        bf_type: 0,
        bf_size: 0,
        bf_reserved1: 0,
        bf_reserved2: 0,
        bf_off_bits: 0,
        bi_size: 0,
        bi_width: 0,
        bi_height: 0,
        bi_planes: 0,
        bi_bit_count: 0,
        bi_compression: 0,
        bi_size_image: 0,
        bi_x_pels_per_meter: 0,
        bi_y_pels_per_meter: 0,
        bi_clr_used: 0,
        bi_clr_important: 0,
    };

    file.read_exact(unsafe { &mut std::slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, std::mem::size_of::<BMPHeader>()) })?;

    let data_size = (3 * header.bi_width * header.bi_height) as usize;
    let mut img = vec![0u8; data_size];
    file.read_exact(&mut img)?;

    Ok((img, header.bi_width, header.bi_height))
}

fn load_density_map(file_name: &str) -> io::Result<()> {
    println!("Loading density map {}", file_name);

    let (data, width, height) = load_bmp(file_name)?;

    println!("Loaded ( {} x {} )", width, height);

    if width != K_IMAGE_SIZE as i32 || height != K_IMAGE_SIZE as i32 {
        eprintln!("ERROR: density map should be {} x {}", K_IMAGE_SIZE, K_IMAGE_SIZE);
        std::process::exit(255);
    }

    unsafe {
        G_DENSITY_MAP = Some(Vec::with_capacity((width * height) as usize));
    }

    let density_map = unsafe { G_DENSITY_MAP.as_mut().unwrap() };

    for y in 0..height {
        for x in 0..width {
            density_map.push(data[(3 * (x + y * width)) as usize] as f32 / 255.0);
        }
    }

    Ok(())
}

fn print_banner() {
    println!("Poisson disk points generator");
    println!("Version 1.7.0");
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com");
    println!("");
    println!("Usage: Poisson [density-map-rgb24.bmp] [--raw-points] [--num-points=<value>] [--square] [--vogel-disk | --jittered-grid | ");
    println!("--hammersley] [--shuffle] [--save-frames] [--save-video[=<skip-frames>]]");
    println!("");
}

fn main() {
    print_banner();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && !args[1].contains("--") {
        if let Err(err) = load_density_map(&args[1]) {
            eprintln!("{}", err);
            return;
        }
    }

    let has_cmd_line_arg = |arg: &str| args.iter().any(|s| s == arg);
    let get_cmd_line_value = |arg: &str, default_value: u32| -> u32 {
        args.iter()
            .filter_map(|s| {
                if s.contains(arg) {
                    s.strip_prefix("--num-points=").and_then(|num| num.parse().ok())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(default_value)
    };
    let get_cmd_line_value_skip_frames = |arg: &str, default_value: u32| -> u32 {
        args.iter()
            .filter_map(|s| {
                if s.contains(arg) {
                    s.strip_prefix("--save-video=").and_then(|num| num.parse().ok())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(default_value)
    };
    let has_cmd_line_arg_prefix = |prefix: &str| args.iter().any(|s| s.starts_with(prefix));

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
        },
    );

    println!("NumPoints = {}", num_points);

    let points: Vec<()> = vec![]; // Placeholder for actual point generation

    // prepare BGR image
    const DATA_SIZE: usize = 3 * K_IMAGE_SIZE * K_IMAGE_SIZE;
    let mut img = vec![0u8; DATA_SIZE];

    if cmd_shuffle {
        println!("Shuffling points...");
        // Placeholder for shuffle function
    }

    let mut avi_writer = if cmd_save_video {
        Some(AVIWriter::new("Points.avi", K_IMAGE_SIZE, K_IMAGE_SIZE, video_skip_frames).unwrap())
    } else {
        None
    };

    for _ in points.iter() {
        // Processing points - placeholders for actual logic
        if avi_writer.is_some() {
            // Placeholder for addFrame call
        }
    }

    if avi_writer.is_some() {
        // Placeholder for final frame flush to video
    }
    avi_writer = None;

    if let Err(err) = save_bmp("Points.bmp", &img, K_IMAGE_SIZE as i32, K_IMAGE_SIZE as i32) {
        eprintln!("{}", err);
    }

    let mut file = File::create("points.txt").unwrap();

    if cmd_raw_points_output {
        writeln!(file, "NumPoints = {}", 0).unwrap();
        // Placeholder for writing points
    } else {
        writeln!(file, "const vec2 points[{}]", 0).unwrap();
        writeln!(file, "{{").unwrap();
        // Placeholder for writing points
        writeln!(file, "}};").unwrap();
    }
}