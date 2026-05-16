use std::collections::LinkedList;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::f32;

struct TPPLPoly {
    points: Vec<TPPLPoint>,
    is_hole: bool,
}

impl TPPLPoly {
    fn new() -> Self {
        TPPLPoly {
            points: Vec::new(),
            is_hole: false,
        }
    }

    fn init(&mut self, numpoints: usize) {
        self.points = vec![TPPLPoint { x: 0.0, y: 0.0 }; numpoints];
    }

    fn set_hole(&mut self, hole: bool) {
        self.is_hole = hole;
    }

    fn get_num_points(&self) -> usize {
        self.points.len()
    }

    fn get_point(&self, index: usize) -> TPPLPoint {
        self.points[index]
    }

    fn is_hole(&self) -> bool {
        self.is_hole
    }
}

#[derive(Copy, Clone, PartialEq)]
struct TPPLPoint {
    x: f32,
    y: f32,
}

impl std::ops::Index<usize> for TPPLPoly {
    type Output = TPPLPoint;
    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Image {
    fn new(width: u32, height: u32) -> Self {
        Image {
            width,
            height,
            data: vec![255; (width * height * 3) as usize],
        }
    }

    fn clear(&mut self, color: Pixel) {
        for i in (0..self.data.len()).step_by(3) {
            self.data[i] = color.r;
            self.data[i + 1] = color.g;
            self.data[i + 2] = color.b;
        }
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Pixel) {
        // Simple Bresenham's line algorithm
        let mut x1 = x1;
        let mut y1 = y1;
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = if dx > dy { dx } else { -dy } / 2;

        loop {
            if x1 >= 0 && x1 < self.width as i32 && y1 >= 0 && y1 < self.height as i32 {
                let index = ((y1 as usize) * self.width as usize + x1 as usize) * 3;
                self.data[index] = color.r;
                self.data[index + 1] = color.g;
                self.data[index + 2] = color.b;
            }

            if x1 == x2 && y1 == y2 {
                break;
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x1 += sx;
            }
            if e2 < dy {
                err += dx;
                y1 += sy;
            }
        }
    }
}

struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

struct ImageIO;

impl ImageIO {
    fn save_image(&self, filename: &str, image: &Image) -> io::Result<()> {
        // This function would save the image data. Placeholder for simplicity.
        let path = format!("{}.ppm", filename);
        let mut file = File::create(&path)?;
        writeln!(file, "P3\n{} {}\n255", image.get_width(), image.get_height())?;
        for pixel in image.data.chunks(3) {
            writeln!(file, "{} {} {}", pixel[0], pixel[1], pixel[2])?;
        }
        Ok(())
    }
}

fn read_poly(filename: &str, poly: &mut TPPLPoly) -> bool {
    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        if let (Some(Ok(numpoints)), Some(Ok(hole))) = (lines.next(), lines.next()) {
            let numpoints: usize = numpoints.parse().unwrap_or(0);
            let hole: i32 = hole.parse().unwrap_or(0);

            poly.init(numpoints);
            poly.set_hole(hole != 0);

            for (i, line) in lines.take(numpoints).enumerate() {
                if let Ok(coords) = line {
                    let coords: Vec<f32> = coords
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if coords.len() == 2 {
                        poly.points[i] = TPPLPoint {
                            x: coords[0],
                            y: coords[1],
                        }
                    }
                }
            }
            return true;
        }
    }
    false
}

fn write_poly(filename: &str, poly: &TPPLPoly) {
    if let Ok(mut file) = File::create(filename) {
        writeln!(file, "{}", poly.get_num_points()).ok();

        writeln!(file, "{}", if poly.is_hole() { 1 } else { 0 }).ok();

        for point in &poly.points {
            writeln!(file, "{} {}", point.x, point.y).ok();
        }
    }
}

fn read_poly_list(filename: &str, polys: &mut LinkedList<TPPLPoly>) -> bool {
    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        if let Some(Ok(numpolys)) = lines.next() {
            let numpolys: usize = numpolys.parse().unwrap_or(0);

            polys.clear();

            for _ in 0..numpolys {
                let mut poly = TPPLPoly::new();
                if !read_poly_line(&mut lines, &mut poly) {
                    return false;
                }
                polys.push_back(poly);
            }
            return true;
        }
    }
    false
}

fn read_poly_line(lines: &mut io::Lines<BufReader<File>>, poly: &mut TPPLPoly) -> bool {
    if let (Some(Ok(numpoints)), Some(Ok(hole))) = (lines.next(), lines.next()) {
        let numpoints: usize = numpoints.parse().unwrap_or(0);
        let hole: i32 = hole.parse().unwrap_or(0);

        poly.init(numpoints);
        poly.set_hole(hole != 0);
        
        for (i, line) in lines.take(numpoints).enumerate() {
            if let Ok(coords) = line {
                let coords: Vec<f32> = coords.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                if coords.len() == 2 {
                    poly.points[i] = TPPLPoint { x: coords[0], y: coords[1] };
                }
            }
        }
        return true;
    }
    false
}

fn write_poly_list(filename: &str, polys: &LinkedList<TPPLPoly>) {
    if let Ok(mut file) = File::create(filename) {
        writeln!(file, "{}", polys.len()).ok();

        for poly in polys.iter() {
            write_poly_to_file(&mut file, poly);
        }
    }
}

fn write_poly_to_file(file: &mut File, poly: &TPPLPoly) {
    writeln!(file, "{}", poly.get_num_points()).ok();

    writeln!(file, "{}", if poly.is_hole() { 1 } else { 0 }).ok();

    for point in &poly.points {
        writeln!(file, "{} {}", point.x, point.y).ok();
    }
}

fn draw_poly(
    img: &mut Image,
    poly: &TPPLPoly,
    xmin: f32,
    xmax: f32,
    ymin: f32,
    ymax: f32,
) {
    let imgmin = TPPLPoint { x: 5.0, y: 5.0 };

    let poly_size_x = xmax - xmin;
    let poly_size_y = ymax - ymin;
    let img_size_x = img.get_width() as f32 - 10.0;
    let img_size_y = img.get_height() as f32 - 10.0;

    let scalex = if poly_size_x > 0.0 {
        img_size_x / poly_size_x
    } else {
        0.0
    };

    let scaley = if poly_size_y > 0.0 {
        img_size_y / poly_size_y
    } else {
        0.0
    };

    let scale = if scalex > 0.0 && scalex < scaley {
        scalex
    } else if scaley > 0.0 {
        scaley
    } else {
        1.0
    };

    let polymin = TPPLPoint { x: xmin, y: ymin };

    for i in 0..poly.get_num_points() {
        let p1 = poly.get_point(i);
        let p2 = poly.get_point((i + 1) % poly.get_num_points());

        let p1img = TPPLPoint {
            x: (p1.x - polymin.x) * scale + imgmin.x,
            y: (p1.y - polymin.y) * scale + imgmin.y,
        };

        let p2img = TPPLPoint {
            x: (p2.x - polymin.x) * scale + imgmin.x,
            y: (p2.y - polymin.y) * scale + imgmin.y,
        };

        img.draw_line(p1img.x as i32, p1img.y as i32, p2img.x as i32, p2img.y as i32, Pixel { r: 0, g: 0, b: 0 });
    }
}

fn draw_poly_list(filename: &str, polys: &LinkedList<TPPLPoly>) {
    let mut img = Image::new(300, 450);
    img.clear(Pixel { r: 255, g: 255, b: 255 });

    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut ymin = f32::MAX;
    let mut ymax = f32::MIN;

    for poly in polys.iter() {
        for i in 0..poly.get_num_points() {
            let point = poly.get_point(i);
            if point.x < xmin {
                xmin = point.x;
            }
            if point.x > xmax {
                xmax = point.x;
            }
            if point.y < ymin {
                ymin = point.y;
            }
            if point.y > ymax {
                ymax = point.y;
            }
        }
    }

    for poly in polys.iter() {
        draw_poly(&mut img, poly, xmin, xmax, ymin, ymax);
    }

    let io = ImageIO;
    io.save_image(filename, &img).unwrap();
}

fn compare_poly(p1: &TPPLPoly, p2: &TPPLPoly) -> bool {
    if p1.get_num_points() != p2.get_num_points() {
        return false;
    }
    for i in 0..p1.get_num_points() {
        if p1[i] != p2[i] {
            return false;
        }
    }
    true
}

fn compare_poly_list(polys1: &LinkedList<TPPLPoly>, polys2: &LinkedList<TPPLPoly>) -> bool {
    if polys1.len() != polys2.len() {
        return false;
    }
    let mut iter1 = polys1.iter();
    let mut iter2 = polys2.iter();
    while let (Some(p1), Some(p2)) = (iter1.next(), iter2.next()) {
        if !compare_poly(p1, p2) {
            return false;
        }
    }
    true
}

fn generate_test_data() {
    let mut pp = TPPLPartition::new();

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list("test_input.txt", &mut testpolys);
    draw_poly_list("test_input.bmp", &testpolys);

    pp.triangulate_ec(&testpolys, &mut result);
    draw_poly_list("test_triangulate_EC.bmp", &result);
    write_poly_list("test_triangulate_EC.txt", &result);

    result.clear();
    expected_result.clear();

    pp.triangulate_opt(testpolys.front().unwrap(), &mut result);
    draw_poly_list("test_triangulate_OPT.bmp", &result);
    write_poly_list("test_triangulate_OPT.txt", &result);

    result.clear();
    expected_result.clear();

    pp.triangulate_mono(&testpolys, &mut result);
    draw_poly_list("test_triangulate_MONO.bmp", &result);
    write_poly_list("test_triangulate_MONO.txt", &result);

    result.clear();
    expected_result.clear();

    pp.convex_partition_hm(&testpolys, &mut result);
    draw_poly_list("test_convexpartition_HM.bmp", &result);
    write_poly_list("test_convexpartition_HM.txt", &result);

    result.clear();
    expected_result.clear();

    pp.convex_partition_opt(testpolys.front().unwrap(), &mut result);
    draw_poly_list("test_convexpartition_OPT.bmp", &result);
    write_poly_list("test_convexpartition_OPT.txt", &result);
}

struct TPPLPartition;

impl TPPLPartition {
    fn new() -> Self {
        TPPLPartition
    }

    fn triangulate_ec(&mut self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Placeholder implementation
    }

    fn triangulate_opt(&mut self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) {
        // Placeholder implementation
    }

    fn triangulate_mono(&mut self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Placeholder implementation
    }

    fn convex_partition_hm(&mut self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Placeholder implementation
    }

    fn convex_partition_opt(&mut self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) {
        // Placeholder implementation
    }
}

fn main() {
    let mut failures = 0;
    let mut pp = TPPLPartition::new();

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list("test_input.txt", &mut testpolys);
    draw_poly_list("test_input.bmp", &testpolys);

    println!("Testing Triangulate_EC: ");
    pp.triangulate_ec(&testpolys, &mut result);
    read_poly_list("test_triangulate_EC.txt", &mut expected_result);
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("tri_ec.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_OPT: ");
    pp.triangulate_opt(testpolys.front().unwrap(), &mut result);
    read_poly_list("test_triangulate_OPT.txt", &mut expected_result);
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("tri_opt.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_MONO: ");
    pp.triangulate_mono(&testpolys, &mut result);
    read_poly_list("test_triangulate_MONO.txt", &mut expected_result);
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("tri_mono.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_HM: ");
    pp.convex_partition_hm(&testpolys, &mut result);
    read_poly_list("test_convexpartition_HM.txt", &mut expected_result);
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("conv_hm.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_OPT: ");
    pp.convex_partition_opt(testpolys.front().unwrap(), &mut result);
    read_poly_list("test_convexpartition_OPT.txt", &mut expected_result);
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("conv_opt.bmp", &result);
}