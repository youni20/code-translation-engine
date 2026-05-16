use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write, Seek};
use std::mem;
use std::string::String;
use std::vec::Vec;

const K_NUM_POINTS_DEFAULT_POISSON: usize = 20000;
const K_NUM_POINTS_DEFAULT_VOGEL: usize = 2000;
const K_NUM_POINTS_DEFAULT_JITTERED: usize = 2500;
const K_IMAGE_SIZE: usize = 512;

// Uint8 array or Vec<u8>
static mut G_DENSITY_MAP: Option<Vec<f32>> = None;

#[repr(packed)]
struct BMPHeader {
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
    file: BufWriter<File>,
    width: usize,
    height: usize,
    fps: u32,
    skip_frames: u32,
    input_frame_count: u32,
    frame_count: u32,
    frame_data_size: u32,
    movi_start: u32,
    row_padding: u32,
    padded_row_size: u32,
    padded_frame_size: u32,
}

impl AVIWriter {
    fn new(file_name: &str, width: usize, height: usize, skip_frames: u32) -> io::Result<AVIWriter> {
        let file = File::create(file_name)?;
        let mut writer = BufWriter::new(file);

        let frame_data_size = (width * height) as u32;
        let row_padding = (4 - (width % 4) as u32) % 4;
        let padded_row_size = width as u32 + row_padding;
        let padded_frame_size = padded_row_size * height as u32;

        println!("Saving video to `{}`", file_name);

        let mut avi_writer = AVIWriter {
            file: writer,
            width,
            height,
            fps: 60,
            skip_frames,
            input_frame_count: 0,
            frame_count: 0,
            frame_data_size,
            movi_start: 0,
            row_padding,
            padded_row_size,
            padded_frame_size,
        };

        avi_writer.write_header()?;
        avi_writer.movi_start = avi_writer.file.stream_position()? as u32;       

        // Start 'movi' LIST
        avi_writer.write_chunk_header("LIST", 0)?;
        avi_writer.write_fourcc("movi")?;

        Ok(avi_writer)
    }

    fn add_frame(&mut self, bgr_data: &[u8], is_last_frame: bool) -> io::Result<bool> {
        if !is_last_frame && (self.input_frame_count % self.skip_frames) != 0 {
            self.input_frame_count += 1;
            return Ok(false);
        }

        self.write_chunk_header("00db", self.padded_frame_size)?;
        let mut row_buffer = vec![0u8; self.padded_row_size as usize];

        for y in 0..self.height {
            for x in 0..self.width {
                row_buffer[x] = bgr_data[(y * self.width + x) * 3];
            }
            self.file.write_all(&row_buffer)?;
        }

        self.frame_count += 1;
        Ok(true)
    }

    fn write_fourcc(&mut self, fourcc: &str) -> io::Result<()> {
        self.file.write_all(fourcc.as_bytes())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.file.write_all(&value.to_le_bytes())
    }

    fn write_chunk_header(&mut self, fourcc: &str, size: u32) -> io::Result<()> {
        self.write_fourcc(fourcc)?;
        self.write_u32(size)
    }

    fn write_header(&mut self) -> io::Result<()> {
        self.write_fourcc("RIFF")?;
        self.write_u32(0)?;
        self.write_fourcc("AVI ")?;
        self.write_fourcc("LIST")?;
        let hdrl_size_pos = self.file.stream_position()? as u32;
        self.write_u32(0)?;
        self.write_fourcc("hdrl")?;
        self.write_main_avi_header(hdrl_size_pos)?;
        Ok(())
    }

    fn write_main_avi_header(&mut self, _hdrl_size_pos: u32) -> io::Result<()> {
        self.write_fourcc("avih")?;
        self.write_u32(56)?; // header size
        self.write_u32(1_000_000 / self.fps)?;
        self.write_u32(self.padded_frame_size * self.fps)?; // dwMaxBytesPerSec
        self.write_u32(0)?; // dwPaddingGranularity
        self.write_u32(0x10)?; // dwFlags (AVIF_HASINDEX)
        self.write_u32(0)?; // dwTotalFrames (placeholder)
        self.write_u32(0)?; // dwInitialFrames
        self.write_u32(1)?; // dwStreams
        self.write_u32(self.padded_frame_size)?; // dwSuggestedBufferSize
        self.write_u32(self.width as u32)?; // dwWidth
        self.write_u32(self.height as u32)?; // dwHeight
        for _ in 0..4 {
            self.write_u32(0)?;
        }
        Ok(())
    }

    fn write_index(&mut self) -> io::Result<()> {
        self.write_fourcc("idx1")?;
        self.write_u32(self.frame_count * 16)?; // index size

        let mut offset = 4; // offset from 'movi' to first frame data

        for _ in 0..self.frame_count {
            self.write_fourcc("00db")?; // chunk ID
            self.write_u32(0x10)?; // flags (AVIIF_KEYFRAME)
            self.write_u32(offset)?; // offset
            self.write_u32(self.padded_frame_size)?; // size

            offset += self.padded_frame_size + 8; // +8 for chunk header
        }
        Ok(())
    }
}

impl Drop for AVIWriter {
    fn drop(&mut self) {
        if let Ok(file_size) = self.file.stream_position() {
            let movi_size = file_size as u32 - self.movi_start - 8;
            if self.write_index().is_ok() {
                if self.file.seek(io::SeekFrom::Start(self.movi_start as u64 + 4)).is_ok()
                    && self.write_u32(movi_size + 4).is_ok()
                    && self.file.seek(io::SeekFrom::Start(4)).is_ok()
                    && self.write_u32(file_size as u32 - 8).is_ok()
                {
                    self.file.flush().ok();
                }
            }
        }
    }
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: usize, height: usize) -> io::Result<()> {
    let header = BMPHeader {
        bf_type: 0x4D42,
        bf_size: (raw_bgr_image.len() + mem::size_of::<BMPHeader>()) as u32,
        bf_reserved1: 0,
        bf_reserved2: 0,
        bf_off_bits: 54,
        bi_size: 40,
        bi_width: width as u32,
        bi_height: height as u32,
        bi_planes: 1,
        bi_bit_count: 24,
        bi_compression: 0,
        bi_size_image: raw_bgr_image.len() as u32,
        bi_x_pels_per_meter: 6000,
        bi_y_pels_per_meter: 6000,
        bi_clr_used: 0,
        bi_clr_important: 0,
    };

    let mut file = BufWriter::new(File::create(file_name)?);
    file.write_all(unsafe {
        std::slice::from_raw_parts((&header as *const BMPHeader) as *const u8, mem::size_of::<BMPHeader>())
    })?;
    file.write_all(raw_bgr_image)?;
    println!("Saved {}", file_name);
    Ok(())
}

fn load_bmp(file_name: &str) -> io::Result<(Vec<u8>, usize, usize)> {
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

    file.read_exact(unsafe {
        std::slice::from_raw_parts_mut((&mut header as *mut BMPHeader) as *mut u8, mem::size_of::<BMPHeader>())
    })?;
    let width = header.bi_width as usize;
    let height = header.bi_height as usize;
    let mut data = vec![0u8; width * height * 3];
    file.read_exact(&mut data)?;
    Ok((data, width, height))
}

fn load_density_map(file_name: &str) {
    println!("Loading density map {}", file_name);

    if let Ok((data, w, h)) = load_bmp(file_name) {
        println!("Loaded ( {} x {} )", w, h);

        if w != K_IMAGE_SIZE || h != K_IMAGE_SIZE {
            println!("ERROR: density map should be {} x {}", K_IMAGE_SIZE, K_IMAGE_SIZE);
            std::process::exit(255);
        }

        unsafe {
            G_DENSITY_MAP = Some(
                data.chunks_exact(3)
                    .map(|chunk| chunk[0] as f32 / 255.0)
                    .collect(),
            )
        };
    }
}

fn print_banner() {
    println!("Poisson disk points generator");
    println!("Version {}", "1.7.0");
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com");
    println!();
    println!(
        "Usage: Poisson [density-map-rgb24.bmp] [--raw-points] \
        [--num-points=<value>] [--square] [--vogel-disk | --jittered-grid | --hammersley] \
        [--shuffle] [--save-frames] [--save-video[=<skip-frames>]]"
    );
    println!();
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && !args[1].starts_with("--") {
        load_density_map(&args[1]);
    }

    let has_cmd_line_arg = |arg: &str| args.iter().any(|a| a == arg);
    
    let get_cmd_line_value = |arg: &str, default_value: usize| {
        for a in &args {
            if a.contains(arg) {
                let mut val = default_value;
                if let Some(equals_pos) = a.find('=') {
                    let value_part = &a[equals_pos + 1..];
                    val = value_part.parse::<usize>().unwrap_or(default_value);
                }
                return val;
            }
        }
        default_value
    };

    let get_cmd_line_value_skip_frames = |arg: &str, default_value: usize| {
        for a in &args {
            if a.contains(arg) {
                if a.starts_with(&format!("{}=", arg)) {
                    let skip = a[arg.len() + 1..].parse::<usize>().unwrap_or(default_value);
                    return skip;
                }
            }
        }
        default_value
    };

    let has_cmd_line_arg_prefix = |prefix: &str| args.iter().any(|a| a.starts_with(prefix));

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

    // Simulating PoissonGenerator namespace
    #[derive(Debug, Clone)]
    struct Point {
        x: f32,
        y: f32,
    }

    struct PoissonGenerator;

    impl PoissonGenerator {
        fn generate_poisson_points(_num_points: usize, _prng: &mut (), _is_square: bool) -> Vec<Point> {
            vec![]
        }
        fn generate_vogel_points(_num_points: usize) -> Vec<Point> {
            vec![]
        }
        fn generate_jittered_grid_points(_num_points: usize, _prng: &mut (), _is_square: bool) -> Vec<Point> {
            vec![]
        }
        fn generate_hammersley_points(_num_points: usize) -> Vec<Point> {
            vec![]
        }
        fn shuffle(points: &mut [Point], _prng: &mut ()) {
            points.reverse(); // placeholder for actual shuffle logic
        }
    }

    let mut prng = ();

    let mut points = if cmd_vogel_disk {
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

    if cmd_shuffle {
        println!("Shuffling points...");
        PoissonGenerator::shuffle(&mut points, &mut prng);
    }

    let mut avi_writer = if cmd_save_video {
        Some(AVIWriter::new("Points.avi", K_IMAGE_SIZE, K_IMAGE_SIZE, video_skip_frames as u32).expect("Failed to create AVI writer"))
    } else {
        None
    };

    let mut frame = 0;
    let mut current_point = 0usize;
    let total_points = points.len();

    for point in points.iter() {
        current_point += 1;
        let x = (point.x * K_IMAGE_SIZE as f32) as isize;
        let y = (point.y * K_IMAGE_SIZE as f32) as isize;
        if x < 0 || y < 0 || x >= K_IMAGE_SIZE as isize || y >= K_IMAGE_SIZE as isize {
            continue;
        }
        
        unsafe {
            if let Some(ref map) = G_DENSITY_MAP {
                let r: f32 = 0.5; // Dummy random value
                let p = map[x as usize + y as usize * K_IMAGE_SIZE];
                if r > p {
                    continue;
                }
            }
        }

        let base = 3 * (x as usize + y as usize * K_IMAGE_SIZE);
        img[base] = 255;
        img[base + 1] = 255;
        img[base + 2] = 255;

        if cmd_save_frames {
            let file_name = format!("pnt{:05}.bmp", frame);
            frame += 1;
            save_bmp(&file_name, &img, K_IMAGE_SIZE, K_IMAGE_SIZE).ok();
        }

        if let Some(ref mut writer) = avi_writer {
            if writer.add_frame(&img, current_point == total_points).expect("Failed to add frame") {
                print!("\rRendering points to video: {}/{}", current_point, total_points);
                io::stdout().flush().ok();
            }
        }
    }

    if let Some(ref mut writer) = avi_writer {
        if writer.add_frame(&img, true).expect("Failed to add final frame") {
            print!("\rRendering points to video: {}/{}", current_point, total_points);
            io::stdout().flush().ok();
        }
    }

    avi_writer = None;

    save_bmp("Points.bmp", &img, K_IMAGE_SIZE, K_IMAGE_SIZE).expect("Failed to save BMP");

    let mut file = File::create("points.txt").expect("Failed to create points.txt");

    if cmd_raw_points_output {
        writeln!(file, "NumPoints = {}", points.len()).expect("Failed to write to points.txt");

        for point in &points {
            writeln!(file, "{} {}", point.x, point.y).expect("Failed to write points to points.txt");
        }
    } else {
        writeln!(file, "const vec2 points[{}]", points.len()).expect("Failed to write to points.txt");
        writeln!(file, "{{").expect("Failed to write to points.txt");
        for point in &points {
            writeln!(file, "\tvec2({:.6}f, {:.6}f),", point.x, point.y).expect("Failed to write points to points.txt");
        }
        writeln!(file, "}};").expect("Failed to write to points.txt");
    }
}