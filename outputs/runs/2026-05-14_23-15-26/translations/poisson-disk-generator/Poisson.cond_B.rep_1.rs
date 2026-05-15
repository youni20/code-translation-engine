use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write, Seek, SeekFrom, Read};
use std::mem::size_of;

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
    bi_x_pels_per_meter: u32,
    bi_y_pels_per_meter: u32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

struct AviWriter {
    file: BufWriter<File>,
    width: usize,
    height: usize,
    fps: usize,
    skip_frames: usize,
    input_frame_count: u32,
    frame_count: u32,
    movi_start: u32,
    padded_frame_size: u32,
}

impl AviWriter {
    fn new(file_name: &str, width: usize, height: usize, skip_frames: usize) -> io::Result<Self> {
        let file = File::create(file_name)?;
        let mut writer = BufWriter::new(file);
        let fps = 60;
        let row_padding = (4 - width % 4) % 4;
        let padded_row_size: usize = width + row_padding;
        let padded_frame_size = padded_row_size * height;

        let movi_start = writer.seek(SeekFrom::Start(size_of::<u32>() as u64))? as u32;

        Ok(AviWriter {
            file: writer,
            width,
            height,
            fps,
            skip_frames,
            input_frame_count: 0,
            frame_count: 0,
            movi_start,
            padded_frame_size: padded_frame_size as u32,
        })
    }

    fn add_frame(&mut self, bgr_data: &[u8], is_last_frame: bool) -> io::Result<bool> {
        if !is_last_frame && (self.input_frame_count % self.skip_frames as u32) != 0 {
            self.input_frame_count += 1;
            return Ok(false);
        }

        self.file.write_all(b"00db")?;
        self.file.write_all(&self.padded_frame_size.to_le_bytes())?;

        let mut row_buffer = vec![0u8; self.width + ((4 - self.width % 4) % 4)];

        for y in 0..self.height {
            for x in 0..self.width {
                row_buffer[x] = bgr_data[(y * self.width + x) * 3];
            }
            self.file.write_all(&row_buffer)?;
        }

        self.frame_count += 1;

        Ok(true)
    }
}

impl Drop for AviWriter {
    fn drop(&mut self) {
        self.file.flush().unwrap(); // Handle errors accordingly in production code
    }
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: usize, height: usize) -> io::Result<()> {
    let image_size = width * height * 3;
    let header = SBmpHeader {
        bf_type: 0x4D42,
        bf_size: image_size as u32 + size_of::<SBmpHeader>() as u32,
        bf_reserved1: 0,
        bf_reserved2: 0,
        bf_off_bits: 54,
        bi_size: 40,
        bi_width: width as u32,
        bi_height: height as u32,
        bi_planes: 1,
        bi_bit_count: 24,
        bi_compression: 0,
        bi_size_image: image_size as u32,
        bi_x_pels_per_meter: 6000,
        bi_y_pels_per_meter: 6000,
        bi_clr_used: 0,
        bi_clr_important: 0,
    };

    let mut file = BufWriter::new(File::create(file_name)?);
    file.write_all(unsafe { std::slice::from_raw_parts((&header as *const SBmpHeader) as *const u8, size_of::<SBmpHeader>()) })?;
    file.write_all(raw_bgr_image)?;

    Ok(())
}

fn load_bmp(file_name: &str) -> io::Result<(Vec<u8>, usize, usize)> {
    let mut file = File::open(file_name)?;
    let mut header = SBmpHeader {
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

    file.read_exact(unsafe { std::slice::from_raw_parts_mut(&mut header as *mut SBmpHeader as *mut u8, size_of::<SBmpHeader>()) })?;

    let width = header.bi_width as usize;
    let height = header.bi_height as usize;
    let data_size = 3 * width * height;

    let mut img_data = vec![0u8; data_size];
    file.read_exact(&mut img_data)?;

    Ok((img_data, width, height))
}

fn print_banner() {
    println!("Poisson disk points generator");
    println!("Version {}", "1.7.0");
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com");
    println!();
    println!("Usage: Poisson [density-map-rgb24.bmp] [--raw-points] [--num-points=<value>] [--square] [--vogel-disk | --jittered-grid | --hammersley] [--shuffle] [--save-frames] [--save-video[=<skip-frames>]]");
    println!();
}

fn main() -> io::Result<()> {
    print_banner();

    let args: Vec<String> = env::args().collect();

    let mut g_density_map: Option<Vec<f32>> = None;

    if args.len() > 1 && !args[1].starts_with("--") {
        let (data, width, height) = load_bmp(&args[1])?;
        if width != K_IMAGE_SIZE || height != K_IMAGE_SIZE {
            println!("ERROR: density map should be {} x {}", K_IMAGE_SIZE, K_IMAGE_SIZE);
            return Ok(());
        }

        let density_map = data
            .chunks_exact(3)
            .map(|chunk| chunk[0] as f32 / 255.0)
            .collect();

        g_density_map = Some(density_map);
    }

    let has_cmd_line_arg = |arg: &str| args.iter().any(|s| s == arg);

    let get_cmd_line_value = |arg: &str, default: usize| -> usize {
        args.iter()
            .filter_map(|s| {
                if s.starts_with(arg) {
                    s.split('=').nth(1).and_then(|v| v.parse::<usize>().ok())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(default)
    };

    let get_cmd_line_value_skip_frames = |arg: &str, default: usize| -> usize {
        args.iter()
            .filter_map(|s| {
                if s.starts_with(arg) {
                    s.split('=').nth(1).and_then(|v| v.parse::<usize>().ok())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(default)
    };

    let has_cmd_line_arg_prefix = |prefix: &str| args.iter().any(|s| s.starts_with(prefix));

    let _cmd_raw_points_output = has_cmd_line_arg("--raw-points");
    let _cmd_square = has_cmd_line_arg("--square");
    let cmd_vogel_disk = has_cmd_line_arg("--vogel-disk");
    let cmd_jittered_grid = has_cmd_line_arg("--jittered-grid");

    let _cmd_hammersley = has_cmd_line_arg("--hammersley");

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

    let mut img: Vec<u8> = vec![0; 3 * K_IMAGE_SIZE * K_IMAGE_SIZE];

    let mut avi_writer = if cmd_save_video {
        Some(AviWriter::new("Points.avi", K_IMAGE_SIZE, K_IMAGE_SIZE, video_skip_frames)?)
    } else {
        None
    };

    if cmd_shuffle {
        println!("Shuffling points...");
    }

    let _frame = 0;
    let _current_point = 0;
    let total_points = 0;

    for _i in 0..total_points {}

    if let Some(writer) = &mut avi_writer {
        writer.add_frame(&img, true)?;
    }

    if cmd_save_frames {
        let file_name = format!("Points.bmp");
        save_bmp(&file_name, &img, K_IMAGE_SIZE, K_IMAGE_SIZE)?;
    }

    avi_writer = None;

    Ok(())
}