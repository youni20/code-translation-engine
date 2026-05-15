use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::f32;
use std::collections::LinkedList;
use std::ops::Add;

#[derive(Clone, PartialEq)]
struct TPPLPoint {
    x: f32,
    y: f32,
}

impl TPPLPoint {
    fn sub(&self, other: &TPPLPoint) -> TPPLPoint {
        TPPLPoint {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn mul(&self, factor: f32) -> TPPLPoint {
        TPPLPoint {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl Add for TPPLPoint {
    type Output = TPPLPoint;

    fn add(self, other: TPPLPoint) -> TPPLPoint {
        TPPLPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

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
        self.points.resize(numpoints, TPPLPoint { x: 0.0, y: 0.0 });
    }

    fn set_hole(&mut self, is_hole: bool) {
        self.is_hole = is_hole;
    }

    fn get_num_points(&self) -> usize {
        self.points.len()
    }

    fn get_point(&self, index: usize) -> &TPPLPoint {
        &self.points[index]
    }
}

struct Image {
    width: u32,
    height: u32,
}

impl Image {
    fn new(width: u32, height: u32) -> Self {
        Image { width, height }
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn clear(&self, _color: Pixel) {
        // Clear the image with a specific color
    }

    fn draw_line(&self, _x1: i32, _y1: i32, _x2: i32, _y2: i32, _color: Pixel) {
        // Draw a line on the image
    }
}

#[derive(Clone, Copy)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

struct ImageIO;

impl ImageIO {
    fn save_image(&self, _filename: &str, _img: &Image) {
        // Save the image to a file
    }
}

struct TPPLPartition;

impl TPPLPartition {
    fn triangulate_ec(_polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Perform triangulation
    }

    fn triangulate_opt(_poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) {
        // Perform optimized triangulation
    }

    fn triangulate_mono(_polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Perform monotone triangulation
    }

    fn convex_partition_hm(_polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Perform convex partition
    }

    fn convex_partition_opt(_poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) {
        // Perform optimized convex partition
    }
}

fn read_poly<R: BufRead>(reader: R, poly: &mut TPPLPoly) {
    let mut lines = reader.lines();
    let numpoints: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    poly.init(numpoints);

    let hole: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();
    poly.set_hole(hole != 0);

    for i in 0..numpoints {
        let line = lines.next().unwrap().unwrap();
        let coords: Vec<&str> = line.trim().split_whitespace().collect();
        let x: f32 = coords[0].parse().unwrap();
        let y: f32 = coords[1].parse().unwrap();
        poly.points[i].x = x;
        poly.points[i].y = y;
    }
}

fn read_poly_from_file(filename: &str, poly: &mut TPPLPoly) -> io::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    read_poly(reader, poly);
    Ok(())
}

fn read_poly_list<R: BufRead>(reader: R, polys: &mut LinkedList<TPPLPoly>) {
    let mut lines = reader.lines();
    let numpolys: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    
    polys.clear();
    
    for _ in 0..numpolys {
        let mut poly = TPPLPoly::new();
        read_poly(&mut BufReader::new(lines.by_ref().next().unwrap().unwrap().as_bytes()), &mut poly);
        polys.push_back(poly);
    }
}

fn read_poly_list_from_file(filename: &str, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    read_poly_list(reader, polys);
    Ok(())
}

fn write_poly<W: Write>(writer: &mut W, poly: &TPPLPoly) -> io::Result<()> {
    writeln!(writer, "{}", poly.get_num_points())?;
    writeln!(writer, "{}", if poly.is_hole { 1 } else { 0 })?;

    for point in &poly.points {
        writeln!(writer, "{} {}", point.x, point.y)?;
    }

    Ok(())
}

fn write_poly_to_file(filename: &str, poly: &TPPLPoly) -> io::Result<()> {
    let mut file = File::create(filename)?;
    write_poly(&mut file, poly)?;
    Ok(())
}

fn write_poly_list<W: Write>(writer: &mut W, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    writeln!(writer, "{}", polys.len())?;
    
    for poly in polys {
        write_poly(writer, poly)?;
    }

    Ok(())
}

fn write_poly_list_to_file(filename: &str, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut file = File::create(filename)?;
    write_poly_list(&mut file, polys)?;
    Ok(())
}

fn draw_poly(img: &Image, poly: &TPPLPoly, xmin: f32, xmax: f32, ymin: f32, ymax: f32) {
    let polymin = TPPLPoint { x: xmin, y: ymin };
    let imgmin = TPPLPoint { x: 5.0, y: 5.0 };

    let poly_size_x = xmax - xmin;
    let poly_size_y = ymax - ymin;
    let img_size_x = img.get_width() as f32 - 10.0;
    let img_size_y = img.get_height() as f32 - 10.0;

    let scalex = if poly_size_x > 0.0 { img_size_x / poly_size_x } else { 0.0 };
    let scaley = if poly_size_y > 0.0 { img_size_y / poly_size_y } else { 0.0 };

    let scale = if scalex > 0.0 && scalex < scaley { scalex } else if scaley > 0.0 { scaley } else { 1.0 };

    for i in 0..poly.get_num_points() {
        let p1 = poly.get_point(i);
        let p2 = poly.get_point((i + 1) % poly.get_num_points());
        let p1img = p1.sub(&polymin).mul(scale) + imgmin.clone();
        let p2img = p2.sub(&polymin).mul(scale) + imgmin.clone();
        img.draw_line(p1img.x as i32, p1img.y as i32, p2img.x as i32, p2img.y as i32, Pixel { r: 0, g: 0, b: 0 });
    }
}

fn draw_poly_to_file(filename: &str, poly: &TPPLPoly) {
    let img = Image::new(500, 500);
    let white = Pixel { r: 255, g: 255, b: 255 };
    img.clear(white);
    let io = ImageIO;

    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut ymin = f32::MAX;
    let mut ymax = f32::MIN;

    for i in 0..poly.get_num_points() {
        if poly.get_point(i).x < xmin {
            xmin = poly.get_point(i).x;
        }
        if poly.get_point(i).x > xmax {
            xmax = poly.get_point(i).x;
        }
        if poly.get_point(i).y < ymin {
            ymin = poly.get_point(i).y;
        }
        if poly.get_point(i).y > ymax {
            ymax = poly.get_point(i).y;
        }
    }

    draw_poly(&img, poly, xmin, xmax, ymin, ymax);
    io.save_image(filename, &img);
}

fn draw_poly_list_to_file(filename: &str, polys: &LinkedList<TPPLPoly>) {
    let img = Image::new(300, 450);
    let white = Pixel { r: 255, g: 255, b: 255 };
    img.clear(white);
    let io = ImageIO;

    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut ymin = f32::MAX;
    let mut ymax = f32::MIN;

    for poly in polys {
        for i in 0..poly.get_num_points() {
            if poly.get_point(i).x < xmin {
                xmin = poly.get_point(i).x;
            }
            if poly.get_point(i).x > xmax {
                xmax = poly.get_point(i).x;
            }
            if poly.get_point(i).y < ymin {
                ymin = poly.get_point(i).y;
            }
            if poly.get_point(i).y > ymax {
                ymax = poly.get_point(i).y;
            }
        }
        // Orientation checking code omitted
    }

    for poly in polys {
        draw_poly(&img, poly, xmin, xmax, ymin, ymax);
    }

    io.save_image(filename, &img);
}

fn compare_poly(p1: &TPPLPoly, p2: &TPPLPoly) -> bool {
    if p1.get_num_points() != p2.get_num_points() {
        return false;
    }
    for i in 0..p1.get_num_points() {
        if p1.points[i] != p2.points[i] {
            return false;
        }
    }
    true
}

fn compare_poly_lists(polys1: &LinkedList<TPPLPoly>, polys2: &LinkedList<TPPLPoly>) -> bool {
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
    let _pp = TPPLPartition;

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list_from_file("test_input.txt", &mut testpolys).unwrap();

    draw_poly_list_to_file("test_input.bmp", &testpolys);

    // Apply partition and triangulation algorithms and store results
    TPPLPartition::triangulate_ec(&testpolys, &mut result);
    draw_poly_list_to_file("test_triangulate_EC.bmp", &result);
    write_poly_list_to_file("test_triangulate_EC.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    if let Some(first) = testpolys.front() {
        TPPLPartition::triangulate_opt(first, &mut result);
        draw_poly_list_to_file("test_triangulate_OPT.bmp", &result);
        write_poly_list_to_file("test_triangulate_OPT.txt", &result).unwrap();
    }

    result.clear();
    expected_result.clear();

    TPPLPartition::triangulate_mono(&testpolys, &mut result);
    draw_poly_list_to_file("test_triangulate_MONO.bmp", &result);
    write_poly_list_to_file("test_triangulate_MONO.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    TPPLPartition::convex_partition_hm(&testpolys, &mut result);
    draw_poly_list_to_file("test_convexpartition_HM.bmp", &result);
    write_poly_list_to_file("test_convexpartition_HM.txt", &result).unwrap();
    
    result.clear();
    expected_result.clear();

    if let Some(first) = testpolys.front() {
        TPPLPartition::convex_partition_opt(first, &mut result);
        draw_poly_list_to_file("test_convexpartition_OPT.bmp", &result);
        write_poly_list_to_file("test_convexpartition_OPT.txt", &result).unwrap();
    }
}

fn main() {
    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list_from_file("test_input.txt", &mut testpolys).unwrap();

    draw_poly_list_to_file("test_input.bmp", &testpolys);

    println!("Testing Triangulate_EC: ");
    TPPLPartition::triangulate_ec(&testpolys, &mut result);
    read_poly_list_from_file("test_triangulate_EC.txt", &mut expected_result).unwrap();
    if compare_poly_lists(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
    }
    draw_poly_list_to_file("tri_ec.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_OPT: ");
    if let Some(first) = testpolys.front() {
        TPPLPartition::triangulate_opt(first, &mut result);
    }
    read_poly_list_from_file("test_triangulate_OPT.txt", &mut expected_result).unwrap();
    if compare_poly_lists(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
    }
    draw_poly_list_to_file("tri_opt.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_MONO: ");
    TPPLPartition::triangulate_mono(&testpolys, &mut result);
    read_poly_list_from_file("test_triangulate_MONO.txt", &mut expected_result).unwrap();
    if compare_poly_lists(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
    }
    draw_poly_list_to_file("tri_mono.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_HM: ");
    TPPLPartition::convex_partition_hm(&testpolys, &mut result);
    read_poly_list_from_file("test_convexpartition_HM.txt", &mut expected_result).unwrap();
    if compare_poly_lists(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
    }
    draw_poly_list_to_file("conv_hm.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_OPT: ");
    if let Some(first) = testpolys.front() {
        TPPLPartition::convex_partition_opt(first, &mut result);
    }
    read_poly_list_from_file("test_convexpartition_OPT.txt", &mut expected_result).unwrap();
    if compare_poly_lists(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
    }
    draw_poly_list_to_file("conv_opt.bmp", &result);
}