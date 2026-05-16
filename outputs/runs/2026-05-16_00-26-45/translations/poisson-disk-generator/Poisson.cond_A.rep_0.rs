use std::env;
use std::fs::File;
use std::io::{self, Write, Seek, SeekFrom, Read};

const K_NUM_POINTS_DEFAULT_POISSON: usize = 20000;
const K_NUM_POINTS_DEFAULT_VOGEL: usize = 2000;
const K_NUM_POINTS_DEFAULT_JITTERED: usize = 2500;
const K_IMAGE_SIZE: usize = 512;

static mut G_DENSITY_MAP: Option<Vec<f32>> = None;

#[repr(C, packed)]
struct SBMPHeader {
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

struct AVIWriter {
    file: File,
    width: usize,
    height: usize,
    fps: usize,
    skip_frames: usize,
    input_frame_count: u32,
    frame_count: u32,
    frame_data_size: usize,
    movi_start: u32,
    row_padding: usize,
    padded_row_size: usize,
    padded_frame_size: usize,
}

impl AVIWriter {
    fn new(file_name: &str, width: usize, height: usize, skip_frames: usize) -> io::Result<AVIWriter> {
        let frame_data_size = width * height;
        let row_padding = (4 - (width % 4)) % 4;
        let padded_row_size = width + row_padding;
        let padded_frame_size = padded_row_size * height;

        println!("\nSaving video to `{}`", file_name);

        let mut file = File::create(file_name)?;

        let movi_start = {
            Self::write_header(&mut file, padded_frame_size, width, height)?;

            file.seek(SeekFrom::Current(4))? as u32
        };

        Ok(AVIWriter {
            file,
            width,
            height,
            fps: 60,
            skip_frames,
            input_frame_count: 0,
            frame_count: 0,
            frame_data_size,
            movi_start,
            row_padding,
            padded_row_size,
            padded_frame_size,
        })
    }

    fn add_frame(&mut self, bgr_data: &[u8], is_last_frame: bool) -> io::Result<bool> {
        if !is_last_frame && (self.input_frame_count % self.skip_frames as u32) != 0 {
            return Ok(false);
        }
        self.input_frame_count += 1;

        AVIWriter::write_chunk_header(&mut self.file, "00db", self.padded_frame_size as u32)?;

        let mut row_buffer: Vec<u8> = vec![0; self.padded_row_size];

        for y in 0..self.height {
            for x in 0..self.width {
                row_buffer[x] = bgr_data[(y * self.width + x) * 3];
            }
            self.file.write_all(&row_buffer)?;
        }

        self.frame_count += 1;

        Ok(true)
    }

    fn finish(mut self) -> io::Result<()> {
        let movi_end = self.file.seek(SeekFrom::Current(0))? as u32;
        let movi_size = movi_end - self.movi_start - 8;

        Self::write_index(&mut self.file, self.frame_count, self.padded_frame_size as u32)?;

        let file_end = self.file.seek(SeekFrom::Current(0))? as u32;

        self.file.seek(SeekFrom::Start(self.movi_start as u64 + 4))?;
        AVIWriter::write_u32(&mut self.file, movi_size + 4)?;

        self.file.seek(SeekFrom::Start(4))?;
        AVIWriter::write_u32(&mut self.file, file_end - 8)?;

        self.file.seek(SeekFrom::Start(48))?;
        AVIWriter::write_u32(&mut self.file, self.frame_count)?;

        self.file.seek(SeekFrom::Start(140))?;
        AVIWriter::write_u32(&mut self.file, self.frame_count)?;

        println!("\nSaved AVI with {} frames", self.frame_count);

        Ok(())
    }

    fn write_header(file: &mut File, padded_frame_size: usize, width: usize, height: usize) -> io::Result<()> {
        AVIWriter::write_fourcc(file, "RIFF")?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_fourcc(file, "AVI ")?;
        AVIWriter::write_fourcc(file, "LIST")?;
        let hdrl_size_pos = file.seek(SeekFrom::Current(0))? as u32;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_fourcc(file, "hdrl")?;
        AVIWriter::write_fourcc(file, "avih")?;
        AVIWriter::write_u32(file, 56)?;
        AVIWriter::write_u32(file, 1000000 / 60)?; // fps = 60
        AVIWriter::write_u32(file, (padded_frame_size * 60) as u32)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, 0x10)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, 1)?;
        AVIWriter::write_u32(file, padded_frame_size as u32)?;
        AVIWriter::write_u32(file, width as u32)?;
        AVIWriter::write_u32(file, height as u32)?;
        for _ in 0..4 {
            AVIWriter::write_u32(file, 0)?;
        }
        AVIWriter::write_fourcc(file, "LIST")?;
        let strl_size_pos = file.seek(SeekFrom::Current(0))? as u32;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_fourcc(file, "strl")?;
        AVIWriter::write_fourcc(file, "strh")?;
        AVIWriter::write_u32(file, 56)?;
        AVIWriter::write_fourcc(file, "vids")?;
        AVIWriter::write_fourcc(file, "DIB ")?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u16(file, 0)?;
        AVIWriter::write_u16(file, 0)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, 1)?;
        AVIWriter::write_u32(file, 60)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, padded_frame_size as u32)?;
        AVIWriter::write_u32(file, 0xFFFFFFFF)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u16(file, 0)?;
        AVIWriter::write_u16(file, 0)?;
        AVIWriter::write_u16(file, width as u16)?;
        AVIWriter::write_u16(file, height as u16)?;

        AVIWriter::write_fourcc(file, "strf")?;
        AVIWriter::write_u32(file, 40 + 256 * 4)?;
        AVIWriter::write_u32(file, 40)?;
        AVIWriter::write_u32(file, width as u32)?;
        AVIWriter::write_u32(file, height as u32)?;
        AVIWriter::write_u16(file, 1)?;
        AVIWriter::write_u16(file, 8)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, padded_frame_size as u32)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, 0)?;
        AVIWriter::write_u32(file, 256)?;
        AVIWriter::write_u32(file, 256)?;

        for i in 0..256 {
            let gray = i as u8;
            file.write_all(&[gray, gray, gray, 0])?;
        }

        let strl_end = file.seek(SeekFrom::Current(0))? as u32;
        file.seek(SeekFrom::Start(strl_size_pos as u64))?;
        AVIWriter::write_u32(file, strl_end - strl_size_pos - 4)?;
        file.seek(SeekFrom::Start(strl_end as u64))?;

        let hdrl_end = file.seek(SeekFrom::Current(0))? as u32;
        file.seek(SeekFrom::Start(hdrl_size_pos as u64))?;
        AVIWriter::write_u32(file, hdrl_end - hdrl_size_pos - 4)?;
        file.seek(SeekFrom::Start(hdrl_end as u64))?;

        Ok(())
    }

    fn write_index(file: &mut File, frame_count: u32, padded_frame_size: u32) -> io::Result<()> {
        AVIWriter::write_fourcc(file, "idx1")?;
        AVIWriter::write_u32(file, frame_count * 16)?;

        let mut offset = 4;

        for _ in 0..frame_count {
            AVIWriter::write_fourcc(file, "00db")?;
            AVIWriter::write_u32(file, 0x10)?;
            AVIWriter::write_u32(file, offset)?;
            AVIWriter::write_u32(file, padded_frame_size)?;

            offset += padded_frame_size + 8;
        }

        Ok(())
    }

    fn write_fourcc(file: &mut File, fourcc: &str) -> io::Result<()> {
        file.write_all(fourcc.as_bytes())?;
        Ok(())
    }

    fn write_u32(file: &mut File, value: u32) -> io::Result<()> {
        file.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_u16(file: &mut File, value: u16) -> io::Result<()> {
        file.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_chunk_header(file: &mut File, fourcc: &str, size: u32) -> io::Result<()> {
        AVIWriter::write_fourcc(file, fourcc)?;
        AVIWriter::write_u32(file, size)?;
        Ok(())
    }
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: usize, height: usize) -> io::Result<()> {
    let mut header: SBMPHeader = unsafe { std::mem::zeroed() };

    let image_size = width * height * 3;

    header.bf_type = 0x4D * 256 + 0x42;
    header.bf_size = (image_size + std::mem::size_of::<SBMPHeader>()) as u32;
    header.bf_off_bits = 0x36;
    header.bi_size = 40;
    header.bi_width = width as u32;
    header.bi_height = height as u32;
    header.bi_planes = 1;
    header.bi_bit_count = 24;
    header.bi_size_image = image_size as u32;
    header.bi_x_pels_per_meter = 6000;
    header.bi_y_pels_per_meter = 6000;

    let mut file = File::create(file_name)?;

    let header_slice: &[u8] = unsafe {
        std::slice::from_raw_parts(&header as *const SBMPHeader as *const u8, std::mem::size_of::<SBMPHeader>())
    };
    file.write_all(header_slice)?;
    file.write_all(raw_bgr_image)?;

    println!("Saved {}", file_name);

    Ok(())
}

fn load_bmp(file_name: &str) -> io::Result<(usize, usize, Vec<u8>)> {
    let mut file = File::open(file_name)?;
    let mut header: SBMPHeader = unsafe { std::mem::zeroed() };

    let header_buffer: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(&mut header as *mut SBMPHeader as *mut u8, std::mem::size_of::<SBMPHeader>())
    };
    file.read_exact(header_buffer)?;

    let out_width = header.bi_width as usize;
    let out_height = header.bi_height as usize;
    let data_size = 3 * out_width * out_height;

    let mut img = vec![0u8; data_size];
    file.read_exact(&mut img)?;

    Ok((out_width, out_height, img))
}

fn load_density_map(file_name: &str) -> io::Result<()> {
    println!("Loading density map {}", file_name);

    let (w, h, data) = load_bmp(file_name)?;

    println!("Loaded ( {} x {} )", w, h);

    if w != K_IMAGE_SIZE || h != K_IMAGE_SIZE {
        println!("ERROR: density map should be {} x {}", K_IMAGE_SIZE, K_IMAGE_SIZE);
        std::process::exit(255);
    }

    let mut density_map = vec![0.0; w * h];

    for y in 0..h {
        for x in 0..w {
            density_map[x + y * w] = data[3 * (x + y * w)] as f32 / 255.0;
        }
    }

    unsafe {
        G_DENSITY_MAP = Some(density_map);
    }

    Ok(())
}

fn print_banner() {
    println!("Poisson disk points generator");
    println!("Version {}", PoissonGenerator::VERSION);
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com");
    println!();
    println!(
        "Usage: Poisson [density-map-rgb24.bmp] [--raw-points] [--num-points=<value>] [--square] \
        [--vogel-disk | --jittered-grid | --hammersley] [--shuffle] [--save-frames] \
        [--save-video[=<skip-frames>]]"
    );
    println!();
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();
    let argc = args.len();

    if argc > 1 && !args[1].contains("--") {
        load_density_map(&args[1]).unwrap();
    }

    let has_cmd_line_arg = |arg: &str| -> bool {
        for arg_test in &args[1..] {
            if arg_test == arg {
                return true;
            }
        }
        false
    };

    let get_cmd_line_value = |arg: &str, default_value: usize| -> usize {
        for arg_test in &args {
            if let Some(pos) = arg_test.find(&arg) {
                if pos == 0 {
                    if let Some(val_str) = arg_test.split('=').nth(1) {
                        if let Ok(value) = val_str.parse() {
                            return value;
                        }
                    }
                }
            }
        }
        default_value
    };

    let get_cmd_line_value_skip_frames = |arg: &str, default_value: usize| -> usize {
        for arg_test in &args {
            if arg_test.starts_with(&arg) {
                if let Some(val_str) = arg_test.split('=').nth(1) {
                    if let Ok(value) = val_str.parse() {
                        return value;
                    }
                }
                return default_value;
            }
        }
        default_value
    };

    let has_cmd_line_arg_prefix = |prefix: &str| -> bool {
        for arg_test in &args {
            if arg_test.starts_with(prefix) {
                return true;
            }
        }
        false
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
        },
    );

    println!("NumPoints = {}", num_points);

    let mut prng = PoissonGenerator::default_prng();
    let points = if cmd_vogel_disk {
        PoissonGenerator::generate_vogel_points(num_points)
    } else if cmd_jittered_grid {
        PoissonGenerator::generate_jittered_grid_points(num_points, &mut prng, !cmd_square)
    } else if cmd_hammersley {
        PoissonGenerator::generate_hammersley_points(num_points)
    } else {
        PoissonGenerator::generate_poisson_points(num_points, &mut prng, !cmd_square)
    };

    let data_size = 3 * K_IMAGE_SIZE * K_IMAGE_SIZE;
    let mut img = vec![0u8; data_size];
    img.fill(0);

    if cmd_shuffle {
        println!("Shuffling points...");
        PoissonGenerator::shuffle(&mut points.clone(), &mut prng);
    }

    let mut avi_writer = if cmd_save_video {
        Some(AVIWriter::new("Points.avi", K_IMAGE_SIZE, K_IMAGE_SIZE, video_skip_frames).unwrap())
    } else {
        None
    };

    let mut frame = 0;
    let total_points = points.len();

    for point in points.iter() {
        let x = (point.x * K_IMAGE_SIZE as f32) as usize;
        let y = (point.y * K_IMAGE_SIZE as f32) as usize;
        if x >= K_IMAGE_SIZE || y >= K_IMAGE_SIZE {
            continue;
        }
        if let Some(ref density_map) = unsafe { &G_DENSITY_MAP } {
            let p = density_map[x + y * K_IMAGE_SIZE];
            if prng.random_f32() > p {
                continue;
            }
        }
        let base = 3 * (x + y * K_IMAGE_SIZE);
        img[base..base + 3].fill(255);

        if cmd_save_frames {
            let file_name = format!("pnt{:05}.bmp", frame);
            save_bmp(&file_name, &img, K_IMAGE_SIZE, K_IMAGE_SIZE).unwrap();
            frame += 1;
        }

        if let Some(writer) = &mut avi_writer {
            if writer.add_frame(&img, points.len() == total_points).unwrap() {
                print!("\rRendering points to video: {}/{}", points.len(), total_points);
                io::stdout().flush().unwrap();
            }
        }
    }

    if let Some(writer) = avi_writer {
        writer.finish().unwrap();
    }

    save_bmp("Points.bmp", &img, K_IMAGE_SIZE, K_IMAGE_SIZE).unwrap();

    let file = File::create("points.txt").unwrap();
    let mut writer = io::BufWriter::new(file);

    if cmd_raw_points_output {
        writeln!(writer, "NumPoints = {}", points.len()).unwrap();

        for point in points {
            writeln!(writer, "{} {}", point.x, point.y).unwrap();
        }
    } else {
        writeln!(writer, "const vec2 points[{}]", points.len()).unwrap();
        writeln!(writer, "{{").unwrap();
        for point in points {
            writeln!(writer, "\tvec2({:.6}f, {:.6}f),", point.x, point.y).unwrap();
        }
        writeln!(writer, "}};").unwrap();
    }
}

mod PoissonGenerator {
    pub const VERSION: &str = "1.7.0";

    #[derive(Clone, Copy)]
    pub struct Point {
        pub x: f32,
        pub y: f32,
    }

    pub struct DefaultPRNG {
        seed: u32,
    }

    impl DefaultPRNG {
        pub fn random_f32(&mut self) -> f32 {
            self.seed ^= self.seed << 13;
            self.seed ^= self.seed >> 17;
            self.seed ^= self.seed << 5;
            (self.seed % 100000) as f32 / 100000.0
        }
    }

    pub fn default_prng() -> DefaultPRNG {
        DefaultPRNG { seed: 123456789 }
    }

    pub fn generate_vogel_points(num_points: usize) -> Vec<Point> {
        // Placeholder implementation
        vec![Point { x: 0.0, y: 0.0 }; num_points]
    }

    pub fn generate_jittered_grid_points(
        num_points: usize,
        _: &mut DefaultPRNG,
        _: bool,
    ) -> Vec<Point> {
        // Placeholder implementation
        vec![Point { x: 0.0, y: 0.0 }; num_points]
    }

    pub fn generate_hammersley_points(num_points: usize) -> Vec<Point> {
        // Placeholder implementation
        vec![Point { x: 0.0, y: 0.0 }; num_points]
    }

    pub fn generate_poisson_points(
        num_points: usize,
        _: &mut DefaultPRNG,
        _: bool,
    ) -> Vec<Point> {
        // Placeholder implementation
        vec![Point { x: 0.0, y: 0.0 }; num_points]
    }

    pub fn shuffle(points: &mut [Point], _: &mut DefaultPRNG) {
        // Placeholder implementation for shuffling points
        points.reverse();
    }
}