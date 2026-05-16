use std::fs::File;
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::mem;
use std::str::FromStr;

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
    width: u32,
    height: u32,
    fps: u32,
    skip_frames: u32,
    input_frame_count: u32,
    frame_count: u32,
    frame_data_size: u32,
    movi_start: u64,
    row_padding: u32,
    padded_row_size: u32,
    padded_frame_size: u32,
}

impl AVIWriter {
    fn new(file_name: &str, width: u32, height: u32, skip_frames: u32) -> io::Result<AVIWriter> {
        let mut file = File::create(file_name)?;
        let fps = 60;
        let frame_data_size = width * height; // grayscale: 1 byte per pixel
        let row_padding = (4 - width % 4) % 4; // pad each row to 4-byte boundary (BMP/AVI requirement)
        let padded_row_size = width + row_padding;
        let padded_frame_size = padded_row_size * height;

        println!("\nSaving video to `{}`", file_name);

        // Write placeholder header (will be updated on close)
        write_header(&mut file, width, height, fps, padded_frame_size)?;

        let movi_start = file.seek(SeekFrom::Current(0))?;

        // Start 'movi' LIST
        write_chunk_header(&mut file, b"LIST", 0)?;
        file.write_all(b"movi")?;

        Ok(AVIWriter {
            file,
            width,
            height,
            fps,
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
        if !is_last_frame && (self.input_frame_count % self.skip_frames) != 0 {
            self.input_frame_count += 1;
            return Ok(false);
        }

        write_chunk_header(&mut self.file, b"00db", self.padded_frame_size)?;

        let mut row_buffer = vec![0u8; self.padded_row_size as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                let src_index = (y * self.width + x) as usize * 3;
                row_buffer[x as usize] = bgr_data[src_index]; // Use the B channel for grayscale
            }
            self.file.write_all(&row_buffer)?;
        }

        self.frame_count += 1;

        Ok(true)
    }
}

impl Drop for AVIWriter {
    fn drop(&mut self) {
        let movi_end = self.file.seek(SeekFrom::Current(0)).unwrap();
        let movi_size = movi_end - self.movi_start - 8;

        write_index(&mut self.file, self.frame_count, 4, self.padded_frame_size).unwrap();

        let file_end = self.file.seek(SeekFrom::Current(0)).unwrap();

        // Update movi LIST size
        self.file.seek(SeekFrom::Start(self.movi_start + 4)).unwrap();
        write_u32(&mut self.file, movi_size as u32 + 4).unwrap();

        // Update RIFF size
        self.file.seek(SeekFrom::Start(4)).unwrap();
        write_u32(&mut self.file, (file_end - 8) as u32).unwrap();

        // Update frame count in header
        self.file.seek(SeekFrom::Start(48)).unwrap();
        write_u32(&mut self.file, self.frame_count).unwrap();

        self.file.seek(SeekFrom::Start(140)).unwrap();
        write_u32(&mut self.file, self.frame_count).unwrap();

        println!("\nSaved AVI with {} frames", self.frame_count);
    }
}

fn write_four_cc<W: Write>(w: &mut W, fourcc: &[u8]) -> io::Result<()> {
    w.write_all(fourcc)
}

fn write_u32<W: Write>(w: &mut W, value: u32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn write_u16<W: Write>(w: &mut W, value: u16) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn write_chunk_header<W: Write>(w: &mut W, fourcc: &[u8], size: u32) -> io::Result<()> {
    write_four_cc(w, fourcc)?;
    write_u32(w, size)
}

fn write_header<W: Write + Seek>(w: &mut W, width: u32, height: u32, fps: u32, padded_frame_size: u32) -> io::Result<()> {
    write_four_cc(w, b"RIFF")?;
    write_u32(w, 0)?; // File size placeholder

    write_four_cc(w, b"AVI ")?;

    write_four_cc(w, b"LIST")?;
    let hdrl_size_pos = w.seek(SeekFrom::Current(0))?;
    write_u32(w, 0)?; // Placeholder

    write_four_cc(w, b"hdrl")?;
    write_four_cc(w, b"avih")?;
    write_u32(w, 56)?;

    write_u32(w, 1_000_000 / fps)?; // dwMicroSecPerFrame
    write_u32(w, padded_frame_size * fps)?; // dwMaxBytesPerSec
    write_u32(w, 0)?; // dwPaddingGranularity
    write_u32(w, 0x10)?; // dwFlags (AVIF_HASINDEX)
    write_u32(w, 0)?; // dwTotalFrames placeholder
    write_u32(w, 0)?; // dwInitialFrames
    write_u32(w, 1)?; // dwStreams
    write_u32(w, padded_frame_size)?; // dwSuggestedBufferSize
    write_u32(w, width)?; // dwWidth
    write_u32(w, height)?; // dwHeight
    for _ in 0..4 {
        write_u32(w, 0)?;
    }

    // Stream LIST
    write_four_cc(w, b"LIST")?;
    let strl_size_pos = w.seek(SeekFrom::Current(0))?;
    write_u32(w, 0)?; // Placeholder

    write_four_cc(w, b"strl")?;

    // Stream header
    write_four_cc(w, b"strh")?;
    write_u32(w, 56)?; // Header size

    write_four_cc(w, b"vids")?; // fccType
    write_four_cc(w, b"DIB ")?; // fccHandler
    write_u32(w, 0)?; // dwFlags
    write_u16(w, 0)?; // wPriority
    write_u16(w, 0)?; // wLanguage
    write_u32(w, 0)?; // dwInitialFrames
    write_u32(w, 1)?; // dwScale
    write_u32(w, fps)?; // dwRate
    write_u32(w, 0)?; // dwStart
    write_u32(w, 0)?; // dwLength placeholder
    write_u32(w, padded_frame_size)?; // dwSuggestedBufferSize
    write_u32(w, 0xFFFFFFFF)?; // dwQuality
    write_u32(w, 0)?; // dwSampleSize

    write_u16(w, 0)?; // rcFrame left
    write_u16(w, 0)?; // rcFrame top
    write_u16(w, width as u16)?; // rcFrame right
    write_u16(w, height as u16)?;

    // Stream format
    write_four_cc(w, b"strf")?;
    write_u32(w, 40 + 256 * 4)?;

    write_u32(w, 40)?; // biSize
    write_u32(w, width)?; // biWidth
    write_u32(w, height)?; // biHeight
    write_u16(w, 1)?; // biPlanes
    write_u16(w, 8)?; // biBitCount
    write_u32(w, 0)?; // biCompression (BI_RGB)
    write_u32(w, padded_frame_size)?; // biSizeImage
    write_i32(w, 0)?; // biXPelsPerMeter
    write_i32(w, 0)?; // biYPelsPerMeter
    write_u32(w, 256)?; // biClrUsed
    write_u32(w, 256)?; // biClrImportant

    // Grayscale palette
    for i in 0..256 {
        let gray = i as u8;
        w.write_all(&[gray, gray, gray, 0])?;
    }

    // Update strl LIST size
    let strl_end = w.seek(SeekFrom::Current(0))?;
    w.seek(SeekFrom::Start(strl_size_pos))?;
    write_u32(w, (strl_end - strl_size_pos) as u32 - 4)?;
    w.seek(SeekFrom::Start(strl_end))?;

    // Update hdrl LIST size
    let hdrl_end = w.seek(SeekFrom::Current(0))?;
    w.seek(SeekFrom::Start(hdrl_size_pos))?;
    write_u32(w, (hdrl_end - hdrl_size_pos) as u32 - 4)?;
    w.seek(SeekFrom::Start(hdrl_end))?;

    Ok(())
}

fn write_index<W: Write>(w: &mut W, frame_count: u32, movi: u32, padded_frame_size: u32) -> io::Result<()> {
    write_four_cc(w, b"idx1")?;
    write_u32(w, frame_count * 16)?;

    let mut offset = movi;

    for _ in 0..frame_count {
        write_four_cc(w, b"00db")?;
        write_u32(w, 0x10)?;
        write_u32(w, offset)?;
        write_u32(w, padded_frame_size)?;

        offset += padded_frame_size + 8;
    }

    Ok(())
}

fn write_i32<W: Write>(w: &mut W, value: i32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn save_bmp(file_name: &str, raw_bgr_image: &[u8], width: i32, height: i32) -> io::Result<()> {
    let image_size = (width * height * 3) as u32;

    let header = BMPHeader {
        bf_type: 0x4D42,
        bf_size: image_size + mem::size_of::<BMPHeader>() as u32,
        bf_reserved1: 0,
        bf_reserved2: 0,
        bf_off_bits: 0x36,
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
    };

    let mut file = File::create(file_name)?;

    file.write_all(any_as_u8_slice(&header))?;
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

    file.read_exact(any_as_u8_slice_mut(&mut header))?;

    let width = header.bi_width;
    let height = header.bi_height;
    let data_size = 3 * width * height;

    let mut img = vec![0; data_size as usize];
    file.read_exact(&mut img)?;

    Ok((img, width, height))
}

fn load_density_map(file_name: &str) -> io::Result<()> {
    println!("Loading density map {}", file_name);

    let (data, w, h) = load_bmp(file_name)?;

    println!("Loaded ( {} x {} )", w, h);

    if w != K_IMAGE_SIZE as i32 || h != K_IMAGE_SIZE as i32 {
        eprintln!("ERROR: density map should be {} x {}", K_IMAGE_SIZE, K_IMAGE_SIZE);
        std::process::exit(255);
    }

    unsafe {
        G_DENSITY_MAP = Some(Vec::with_capacity((w * h) as usize));

        for y in 0..h {
            for x in 0..w {
                let idx = (x + y * w) as usize * 3;
                (*G_DENSITY_MAP.as_mut().unwrap()).push(data[idx] as f32 / 255.0);
            }
        }
    }

    Ok(())
}

fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((p as *const T) as *const u8, mem::size_of::<T>()) }
}

fn any_as_u8_slice_mut<T: Sized>(p: &mut T) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut((p as *mut T) as *mut u8, mem::size_of::<T>()) }
}

fn print_banner() {
    println!("Poisson disk points generator");
    println!("Version {}", "1.7.0"); // assuming PoissonGenerator::Version
    println!("Sergey Kosarevsky, 2014-2026");
    println!("support@linderdaum.com http://www.linderdaum.com http://blog.linderdaum.com");
    println!();
    println!("Usage: Poisson [density-map-rgb24.bmp] [--raw-points] [--num-points=<value>] [--square] [--vogel-disk | --jittered-grid | \
             --hammersley] [--shuffle] [--save-frames] [--save-video[=<skip-frames>]]");
    println!();
}

// Assume YourPointType is defined as follows:
struct YourPointType {
    x: f32,
    y: f32,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    print_banner();

    if args.len() > 1 && !args[1].starts_with("--") {
        load_density_map(&args[1]).unwrap();
    }

    let has_cmd_line_arg = |arg: &str| -> bool {
        args.iter().skip(1).any(|a| a == arg)
    };

    let get_cmd_line_value = |arg: &str, default_value: u32| -> u32 {
        args.iter().skip(1).find_map(|a| {
            if a.starts_with(arg) {
                a.split('=').nth(1).and_then(|s| u32::from_str(s).ok())
            } else {
                None
            }
        }).unwrap_or(default_value)
    };

    let get_cmd_line_value_skip_frames = |arg: &str, default_value: u32| -> u32 {
        args.iter().skip(1).find_map(|a| {
            if let Some(equals_pos) = a.find('=') {
                if &a[..equals_pos] == arg {
                    u32::from_str(&a[(equals_pos + 1)..]).ok()
                } else {
                    None
                }
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
        "--num-points=", if cmd_vogel_disk { K_NUM_POINTS_DEFAULT_VOGEL } else if cmd_jittered_grid { K_NUM_POINTS_DEFAULT_JITTERED } else { K_NUM_POINTS_DEFAULT_POISSON }
    );

    println!("NumPoints = {}", num_points);

    // Placeholder for the PoissonGenerator calls
    // You must define the methods such as generate_vogel_points, generate_jittered_grid_points, etc.

    let mut prng = (); // Placeholder for prng, depending on the actual implementation of PoissonGenerator.

    let points: Vec<YourPointType> = if cmd_vogel_disk {
        Vec::new() // Placeholder
    } else if cmd_jittered_grid {
        Vec::new() // Placeholder
    } else if cmd_hammersley {
        Vec::new() // Placeholder
    } else {
        Vec::new() // Placeholder
    };

    let data_size = 3 * K_IMAGE_SIZE * K_IMAGE_SIZE;
    let mut img = vec![0u8; data_size];

    if cmd_shuffle {
        println!("Shuffling points...");
        // Placeholder for the shuffle function
    }

    let mut avi_writer = if cmd_save_video {
        Some(AVIWriter::new("Points.avi", K_IMAGE_SIZE as u32, K_IMAGE_SIZE as u32, video_skip_frames).unwrap())
    } else {
        None
    };

    let mut frame = 0;
    let mut current_point = 0;
    let total_points = points.len();

    for point in &points {
        current_point += 1;
        let x = (point.x * K_IMAGE_SIZE as f32) as i32;
        let y = (point.y * K_IMAGE_SIZE as f32) as i32;
        if x < 0 || y < 0 || x >= K_IMAGE_SIZE as i32 || y >= K_IMAGE_SIZE as i32 {
            continue;
        }

        unsafe {
            if let Some(density_map) = &G_DENSITY_MAP {
                let r: f32 = 0.0; // Placeholder for random_float
                let p = density_map[(x + y * K_IMAGE_SIZE as i32) as usize];
                if r > p {
                    continue;
                }
            }
        }

        let base = 3 * (x + y * K_IMAGE_SIZE as i32) as usize;
        img[base..base+3].fill(255);

        if cmd_save_frames {
            let file_name = format!("pnt{:05}.bmp", frame);
            save_bmp(&file_name, &img, K_IMAGE_SIZE as i32, K_IMAGE_SIZE as i32).unwrap();
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

    avi_writer = None;

    save_bmp("Points.bmp", &img, K_IMAGE_SIZE as i32, K_IMAGE_SIZE as i32).unwrap();

    let mut file = File::create("points.txt").unwrap();

    if cmd_raw_points_output {
        writeln!(file, "NumPoints = {}", points.len()).unwrap();
        for p in &points {
            writeln!(file, "{} {}", p.x, p.y).unwrap();
        }
    } else {
        writeln!(file, "const vec2 points[{}]", points.len()).unwrap();
        writeln!(file, "{{").unwrap();
        file.write_all(format!("{:.6}", points.iter().map(|p| format!("vec2({}f, {}f),", p.x, p.y)).collect::<Vec<_>>().join("\n")).as_bytes()).unwrap();
        writeln!(file, "}};").unwrap();
    }
}