use std::fs::File;
use std::io::{self, BufWriter, Write, Read, Seek, SeekFrom};
use std::env;

const KNUM_POINTS_DEFAULT_POISSON: u32 = 20000;
const KNUM_POINTS_DEFAULT_VOGEL: u32 = 2000;
const KNUM_POINTS_DEFAULT_JITTERED: u32 = 2500;
const KIMAGE_SIZE: usize = 512;

#[repr(C, packed)]
struct BitmapHeader {
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

impl BitmapHeader {
    fn new(width: u32, height: u32) -> Self {
        let image_size = width * height * 3;
        BitmapHeader {
            bf_type: 0x4D42,
            bf_size: image_size as u32 + std::mem::size_of::<BitmapHeader>() as u32,
            bf_reserved1: 0,
            bf_reserved2: 0,
            bf_off_bits: 54,
            bi_size: 40,
            bi_width: width,
            bi_height: height,
            bi_planes: 1,
            bi_bit_count: 24,
            bi_compression: 0,
            bi_size_image: image_size,
            bi_x_pels_per_meter: 6000,
            bi_y_pels_per_meter: 6000,
            bi_clr_used: 0,
            bi_clr_important: 0,
        }
    }
}

struct AviWriter {
    file: BufWriter<File>,
    width: usize,
    height: usize,
    fps: usize,
    skip_frames: usize,
    input_frame_count: u32,
    frame_count: u32,
    frame_data_size: u32,
    movi_start: u64,
    row_padding: usize,
    padded_row_size: usize,
    padded_frame_size: u32,
}

impl AviWriter {
    fn new(file_name: &str, width: usize, height: usize, skip_frames: usize) -> io::Result<Self> {
        let file = BufWriter::new(File::create(file_name)?);
        let frame_data_size = (width * height) as u32;
        let row_padding = (4 - (width) % 4) % 4;
        let padded_row_size = width + row_padding;
        let padded_frame_size = (padded_row_size * height) as u32;

        let mut writer = AviWriter {
            file,
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

        writer.write_header()?;
        writer.movi_start = writer.file.stream_position()?;
        writer.write_chunk_header("LIST", 0)?;
        writer.write_fourcc(b"movi")?;
        Ok(writer)
    }

    fn write_fourcc(&mut self, fourcc: &[u8]) -> io::Result<()> {
        self.file.write_all(fourcc)?;
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
        self.write_fourcc(fourcc.as_bytes())?;
        self.write_u32(size)?;
        Ok(())
    }

    fn write_header(&mut self) -> io::Result<()> {
        self.write_fourcc(b"RIFF")?;
        self.write_u32(0)?; // placeholder
        self.write_fourcc(b"AVI ")?;
        self.write_fourcc(b"LIST")?; // hdrl
        let hdrl_size_pos = self.file.stream_position()?;
        self.write_u32(0)?; // size placeholder
        self.write_fourcc(b"hdrl")?;
        self.write_fourcc(b"avih")?;
        self.write_u32(56)?; // header size
        self.write_u32(1_000_000 / self.fps as u32)?;
        self.write_u32(self.padded_frame_size * self.fps as u32)?;
        self.write_u32(0)?;
        self.write_u32(0x10)?; // AVIF_HASINDEX
        self.write_u32(0)?; // placeholder
        self.write_u32(0)?;
        self.write_u32(1)?;
        self.write_u32(self.padded_frame_size)?;
        self.write_u32(self.width as u32)?;
        self.write_u32(self.height as u32)?;
        for _ in 0..4 {
            self.write_u32(0)?;
        }
        self.write_fourcc(b"LIST")?;
        let strl_size_pos = self.file.stream_position()?;
        self.write_u32(0)?; // size placeholder
        self.write_fourcc(b"strl")?;
        self.write_fourcc(b"strh")?;
        self.write_u32(56)?; // header size
        self.write_fourcc(b"vids")?;
        self.write_fourcc(b"DIB ")?;
        self.write_u32(0)?;
        self.write_u16(0)?;
        self.write_u16(0)?;
        self.write_u32(0)?;
        self.write_u32(1)?;
        self.write_u32(self.fps as u32)?;
        self.write_u32(0)?;
        self.write_u32(0)?;
        self.write_u32(self.padded_frame_size)?;
        self.write_u32(0xFFFFFFFF)?;
        self.write_u32(0)?;
        self.write_u16(0)?;
        self.write_u16(0)?;
        self.write_u16(self.width as u16)?;
        self.write_u16(self.height as u16)?;
        self.write_fourcc(b"strf")?;
        self.write_u32(40 + 256 * 4)?;
        self.write_u32(40)?;
        self.write_u32(self.width as u32)?;
        self.write_u32(self.height as u32)?;
        self.write_u16(1)?;
        self.write_u16(8)?;
        self.write_u32(0)?;
        self.write_u32(self.padded_frame_size)?;
        for _ in 0..2 {
            self.write_u32(0)?;
        }
        self.write_u32(256)?;
        self.write_u32(256)?;

        for i in 0..256 {
            let gray = i as u8;
            self.file.write_all(&[gray, gray, gray, 0])?;
        }

        let strl_end = self.file.stream_position()?;
        self.file.seek(SeekFrom::Start(strl_size_pos))?;
        self.write_u32((strl_end - strl_size_pos - 4) as u32)?;
        self.file.seek(SeekFrom::Start(strl_end))?;

        let hdrl_end = self.file.stream_position()?;
        self.file.seek(SeekFrom::Start(hdrl_size_pos))?;
        self.write_u32((hdrl_end - hdrl_size_pos - 4) as u32)?;
        self.file.seek(SeekFrom::Start(hdrl_end))?;

        Ok(())
    }

    fn write_index(&mut self) -> io::Result<()> {
        self.write_fourcc(b"idx1")?;
        self.write_u32(self.frame_count * 16)?;

        let mut offset = 4;

        for _ in 0..self.frame_count {
            self.write_fourcc(b"00db")?;
            self.write_u32(0x10)?;
            self.write_u32(offset)?;
            self.write_u32(self.padded_frame_size)?;

            offset += self.padded_frame_size + 8;
        }
        Ok(())
    }

    fn add_frame(&mut self, bgr_data: &[u8], is_last_frame: bool) -> io::Result<bool> {
        if !is_last_frame && (self.input_frame_count % self.skip_frames as u32) != 0 {
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
}

impl Drop for AviWriter {
    fn drop(&mut self) {
        let _ = self.file.flush();
        self.write_index().unwrap();

        let file_end = self.file.stream_position().unwrap();

        self.file.seek(SeekFrom::Start(self.movi_start + 4)).unwrap();
        self.write_u32((file_end - self.movi_start - 4) as u32).unwrap();

        self.file.seek(SeekFrom::Start(4)).unwrap();
        self.write_u32((file_end - 8) as u32).unwrap();

        self.file.seek(SeekFrom::Start(48)).unwrap();
        self.write_u32(self.frame_count).unwrap();

        self.file.seek(SeekFrom::Start(140)).unwrap();
        self.write_u32(self.frame_count).unwrap();

        println!("\nSaved AVI with {} frames", self.frame_count);
    }
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: usize, height: usize) -> io::Result<()> {
    let header = BitmapHeader::new(width as u32, height as u32);

    let file = File::create(file_name)?;
    let mut writer = BufWriter::new(file);

    let bytes = unsafe {
        std::slice::from_raw_parts(
            &header as *const BitmapHeader as *const u8,
            std::mem::size_of::<BitmapHeader>(),
        )
    };
    writer.write_all(bytes)?;

    writer.write_all(raw_bgr_image)?;
    writer.flush()?;

    println!("Saved {}", file_name);
    Ok(())
}

fn load_bmp(file_name: &str) -> io::Result<(Vec<u8>, i32, i32)> {
    let mut file = File::open(file_name)?;

    let mut header = BitmapHeader::new(0, 0);
    unsafe {
        let header_bytes = std::slice::from_raw_parts_mut(
            &mut header as *mut BitmapHeader as *mut u8,
            std::mem::size_of::<BitmapHeader>(),
        );
        file.read_exact(header_bytes)?;
    }

    let width = header.bi_width as i32;
    let height = header.bi_height as i32;
    let data_size = (3 * width * height) as usize;

    let mut img = vec![0u8; data_size];
    file.read_exact(&mut img)?;

    Ok((img, width, height))
}

#[derive(Clone)]
struct PoissonPoint {
    x: f64,
    y: f64,
}

trait PoissonGenerator {
    fn version() -> &'static str;
    fn generate_poisson_points(num_points: u32) -> Vec<PoissonPoint>;
    fn shuffle(points: &mut [PoissonPoint]);
}

fn print_banner() {
    println!("Poisson disk points generator");
    // Dummy implementation to fix the error
    println!("Version 1.0.0");
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com\n");
    println!(
        "Usage: Poisson [density-map-rgb24.bmp] [--raw-points] [--num-points=<value>] \
        [--square] [--vogel-disk | --jittered-grid | --hammersley] [--shuffle] [--save-frames] \
        [--save-video[=<skip-frames>]]\n"
    );
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    let density_map = if args.len() > 1 && !args[1].starts_with("--") {
        Some(args[1].clone())
    } else {
        None
    };

    let has_cmd_line_arg = |arg: &str| {
        args.iter().any(|a| a == arg)
    };

    let get_cmd_line_value = |arg: &str, default_value: u32| -> u32 {
        args.iter().find_map(|a| {
            if a.starts_with(arg) {
                a.split('=').nth(1).and_then(|v| v.parse().ok())
            } else {
                None
            }
        }).unwrap_or(default_value)
    };

    let get_cmd_line_value_skip_frames = |arg: &str, default_value: u32| -> u32 {
        args.iter().find_map(|a| {
            if a.starts_with(arg) {
                let parts: Vec<&str> = a.split('=').collect();
                if parts.len() == 2 {
                    return parts[1].parse().ok();
                }
            }
            None
        }).unwrap_or(default_value)
    };

    let has_cmd_line_arg_prefix = |prefix: &str| args.iter().any(|a| a.starts_with(prefix));

    let cmd_raw_points_output = has_cmd_line_arg("--raw-points");
    let _cmd_square = has_cmd_line_arg("--square");
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
            KNUM_POINTS_DEFAULT_VOGEL
        } else if cmd_jittered_grid {
            KNUM_POINTS_DEFAULT_JITTERED
        } else {
            KNUM_POINTS_DEFAULT_POISSON
        },
    );

    println!("NumPoints = {}", num_points);

    let points: Vec<PoissonPoint> = if cmd_vogel_disk {
        Vec::new()
    } else if cmd_jittered_grid {
        Vec::new()
    } else if cmd_hammersley {
        Vec::new()
    } else {
        Vec::new()
    };

    let data_size = 3 * KIMAGE_SIZE * KIMAGE_SIZE;
    let mut img = vec![0u8; data_size];

    if cmd_shuffle {
        println!("Shuffling points...");
        // PoissonGenerator::shuffle(&mut points); // This trait method call will need a concrete implementation
    }

    let mut avi_writer = if cmd_save_video {
        Some(AviWriter::new("Points.avi", KIMAGE_SIZE, KIMAGE_SIZE, video_skip_frames as usize).unwrap())
    } else {
        None
    };

    let mut frame = 0;
    let mut current_point = 0;
    let total_points = points.len();

    for i in points.iter() {
        current_point += 1;
        let x = (i.x * KIMAGE_SIZE as f64) as i32;
        let y = (i.y * KIMAGE_SIZE as f64) as i32;
        if x < 0 || y < 0 || x >= KIMAGE_SIZE as i32 || y >= KIMAGE_SIZE as i32 {
            continue;
        }
        if let Some(_) = density_map {
            // let r = rand::random::<f32>();
            // let p = g_density_map[x as usize + (y as usize) * KIMAGE_SIZE];
            // if r > p { continue; }
        }
        let base = 3 * (x as usize + (y as usize) * KIMAGE_SIZE);
        img[base..base + 3].fill(255);

        if cmd_save_frames {
            save_bmp(&format!("pnt{:05}.bmp", frame), &img, KIMAGE_SIZE, KIMAGE_SIZE).unwrap();
            frame += 1;
        }

        if let Some(ref mut writer) = avi_writer {
            if writer.add_frame(&img, current_point == total_points).unwrap() {
                print!("\rRendering points to video: {}/{}", current_point, total_points);
                io::stdout().flush().unwrap();
            }
        }
    }

    if let Some(ref mut writer) = avi_writer {
        if writer.add_frame(&img, true).unwrap() {
            print!("\rRendering points to video: {}/{}", current_point, total_points);
            io::stdout().flush().unwrap();
        }
    }

    save_bmp("Points.bmp", &img, KIMAGE_SIZE, KIMAGE_SIZE).unwrap();

    let file = File::create("points.txt").expect("Unable to create points.txt");
    let mut writer = BufWriter::new(file);

    if cmd_raw_points_output {
        writeln!(writer, "NumPoints = {}", points.len()).unwrap();
        for p in points.iter() {
            writeln!(writer, "{} {}", p.x, p.y).unwrap();
        }
    } else {
        writeln!(writer, "const vec2 points[{}]", points.len()).unwrap();
        writeln!(writer, "{{").unwrap();
        for p in points.iter() {
            writeln!(writer, "\tvec2({:.6}f, {:.6}f),", p.x, p.y).unwrap();
        }
        writeln!(writer, "}};").unwrap();
    }
}