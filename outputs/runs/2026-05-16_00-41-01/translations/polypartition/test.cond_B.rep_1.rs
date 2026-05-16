use std::fs::File;
use std::io::{self, BufRead, Write};
use std::f32;
use std::collections::LinkedList;

#[derive(Clone)]
struct TPPLPoint {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct TPPLPoly {
    points: Vec<TPPLPoint>,
    is_hole: bool,
}

impl TPPLPoly {
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
        self.points[index].clone()
    }
}

impl std::ops::Index<usize> for TPPLPoly {
    type Output = TPPLPoint;
    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

impl std::ops::IndexMut<usize> for TPPLPoly {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.points[index]
    }
}

fn read_poly<R: BufRead>(reader: &mut R, poly: &mut TPPLPoly) -> io::Result<()> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let numpoints: usize = line.trim().parse().unwrap();
    poly.init(numpoints);

    line.clear();
    reader.read_line(&mut line)?;
    let hole: i32 = line.trim().parse().unwrap();
    if hole != 0 {
        poly.set_hole(true);
    }

    for i in 0..numpoints {
        line.clear();
        reader.read_line(&mut line)?;
        let mut iter = line.trim().split_whitespace();
        let x: f32 = iter.next().unwrap().parse().unwrap();
        let y: f32 = iter.next().unwrap().parse().unwrap();
        poly[i].x = x;
        poly[i].y = y;
    }
    Ok(())
}

fn read_poly_file(filename: &str, poly: &mut TPPLPoly) -> io::Result<()> {
    let file = File::open(filename)?;
    let mut reader = io::BufReader::new(file);
    read_poly(&mut reader, poly)
}

fn read_poly_list<R: BufRead>(reader: &mut R, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    polys.clear();

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let numpolys: usize = line.trim().parse().unwrap();
    for _ in 0..numpolys {
        let mut poly = TPPLPoly { points: Vec::new(), is_hole: false };
        read_poly(reader, &mut poly)?;
        polys.push_back(poly);
    }
    Ok(())
}

fn read_poly_list_file(filename: &str, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    let file = File::open(filename)?;
    let mut reader = io::BufReader::new(file);
    read_poly_list(&mut reader, polys)
}

fn write_poly<W: Write>(writer: &mut W, poly: &TPPLPoly) -> io::Result<()> {
    writeln!(writer, "{}", poly.get_num_points())?;
    writeln!(writer, "{}", if poly.is_hole { 1 } else { 0 })?;

    for point in &poly.points {
        writeln!(writer, "{} {}", point.x, point.y)?;
    }
    Ok(())
}

fn write_poly_file(filename: &str, poly: &TPPLPoly) -> io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = io::BufWriter::new(file);
    write_poly(&mut writer, poly)
}

fn write_poly_list<W: Write>(writer: &mut W, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    writeln!(writer, "{}", polys.len())?;

    for poly in polys {
        write_poly(writer, poly)?;
    }
    Ok(())
}

fn write_poly_list_file(filename: &str, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = io::BufWriter::new(file);
    write_poly_list(&mut writer, polys)
}

#[derive(Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

struct Image {
    width: usize,
    height: usize,
    data: Vec<Pixel>,
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        Image {
            width,
            height,
            data: vec![Pixel { r: 255, g: 255, b: 255 }; width * height],
        }
    }

    fn clear(&mut self, color: Pixel) {
        for pixel in &mut self.data {
            *pixel = color.clone();
        }
    }

    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }

    fn draw_line(&mut self, _x1: usize, _y1: usize, _x2: usize, _y2: usize, _color: Pixel) {
        // Drawing a line can be implemented here if needed
    }
}

struct ImageIO;

impl ImageIO {
    fn save_image(&self, _filename: &str, _img: &Image) {
        // Saving image logic could be added here
    }
}

fn draw_poly(img: &mut Image, poly: &TPPLPoly, xmin: f32, xmax: f32, ymin: f32, ymax: f32) {
    let img_width = img.get_width() as f32;
    let img_height = img.get_height() as f32;
    let img_min = TPPLPoint { x: 5.0, y: 5.0 };

    let poly_size_x = xmax - xmin;
    let poly_size_y = ymax - ymin;

    let img_size_x = img_width - 10.0;
    let img_size_y = img_height - 10.0;

    let mut scale = 1.0;
    if poly_size_x > 0.0 {
        scale = img_size_x / poly_size_x;
    }
    if poly_size_y > 0.0 && (img_size_y / poly_size_y) < scale {
        scale = img_size_y / poly_size_y;
    }

    let color = Pixel { r: 0, g: 0, b: 0 };
    for i in 0..poly.get_num_points() {
        let p1 = poly.get_point(i);
        let p2 = poly.get_point((i + 1) % poly.get_num_points());

        let p1_img = TPPLPoint {
            x: (p1.x - xmin) * scale + img_min.x,
            y: (p1.y - ymin) * scale + img_min.y,
        };
        let p2_img = TPPLPoint {
            x: (p2.x - xmin) * scale + img_min.x,
            y: (p2.y - ymin) * scale + img_min.y,
        };

        img.draw_line(
            p1_img.x as usize, 
            p1_img.y as usize, 
            p2_img.x as usize, 
            p2_img.y as usize,
            color.clone()
        );
    }
}

fn draw_poly_image(filename: &str, poly: &TPPLPoly) {
    let mut img = Image::new(500, 500);
    let white = Pixel { r: 255, g: 255, b: 255 };
    img.clear(white);
    let io = ImageIO;

    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut ymin = f32::MAX;
    let mut ymax = f32::MIN;
    for i in 0..poly.get_num_points() {
        let point = poly.get_point(i);
        if point.x < xmin { xmin = point.x; }
        if point.x > xmax { xmax = point.x; }
        if point.y < ymin { ymin = point.y; }
        if point.y > ymax { ymax = point.y; }
    }

    draw_poly(&mut img, poly, xmin, xmax, ymin, ymax);
    io.save_image(filename, &img);
}

fn draw_poly_list(filename: &str, polys: &LinkedList<TPPLPoly>) {
    let mut img = Image::new(300, 450);
    let white = Pixel { r: 255, g: 255, b: 255 };
    img.clear(white);

    let io = ImageIO;
    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut ymin = f32::MAX;
    let mut ymax = f32::MIN;

    for poly in polys {
        for i in 0..poly.get_num_points() {
            let point = poly.get_point(i);
            if point.x < xmin { xmin = point.x; }
            if point.x > xmax { xmax = point.x; }
            if point.y < ymin { ymin = point.y; }
            if point.y > ymax { ymax = point.y; }
        }
    }

    for poly in polys {
        draw_poly(&mut img, poly, xmin, xmax, ymin, ymax);
    }
    
    io.save_image(filename, &img);
}

fn compare_poly(p1: &TPPLPoly, p2: &TPPLPoly) -> bool {
    if p1.get_num_points() != p2.get_num_points() {
        return false;
    }
    p1.points.iter().zip(&p2.points).all(|(a, b)| a.x == b.x && a.y == b.y)
}

fn compare_poly_list(polys1: &LinkedList<TPPLPoly>, polys2: &LinkedList<TPPLPoly>) -> bool {
    if polys1.len() != polys2.len() {
        return false;
    }

    polys1.iter().zip(polys2.iter()).all(|(p1, p2)| compare_poly(p1, p2))
}

struct TPPLPartition;

impl TPPLPartition {
    fn triangulate_ec(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Triangulation logic
    }

    fn triangulate_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) {
        // Optimal triangulation logic
    }

    fn triangulate_mono(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // Monotone triangulation logic
    }

    fn convex_partition_hm(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) {
        // HM convex partitioning logic
    }
    
    fn convex_partition_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) {
        // Optimal convex partitioning logic
    }
}

fn generate_test_data() {
    let pp = TPPLPartition;
    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list_file("test_input.txt", &mut testpolys).unwrap();
    draw_poly_list("test_input.bmp", &testpolys);

    pp.triangulate_ec(&testpolys, &mut result);
    draw_poly_list("test_triangulate_EC.bmp", &result);
    write_poly_list_file("test_triangulate_EC.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.triangulate_opt(&testpolys.front().unwrap(), &mut result);
    draw_poly_list("test_triangulate_OPT.bmp", &result);
    write_poly_list_file("test_triangulate_OPT.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.triangulate_mono(&testpolys, &mut result);
    draw_poly_list("test_triangulate_MONO.bmp", &result);
    write_poly_list_file("test_triangulate_MONO.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.convex_partition_hm(&testpolys, &mut result);
    draw_poly_list("test_convexpartition_HM.bmp", &result);
    write_poly_list_file("test_convexpartition_HM.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.convex_partition_opt(&testpolys.front().unwrap(), &mut result);
    draw_poly_list("test_convexpartition_OPT.bmp", &result);
    write_poly_list_file("test_convexpartition_OPT.txt", &result).unwrap();
}

fn main() -> Result<(), i32> {
    let mut failures = 0;
    let pp = TPPLPartition;

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list_file("test_input.txt", &mut testpolys).unwrap();
    draw_poly_list("test_input.bmp", &testpolys);

    println!("Testing Triangulate_EC: ");
    pp.triangulate_ec(&testpolys, &mut result);
    read_poly_list_file("test_triangulate_EC.txt", &mut expected_result).unwrap();
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
    pp.triangulate_opt(&testpolys.front().unwrap(), &mut result);
    read_poly_list_file("test_triangulate_OPT.txt", &mut expected_result).unwrap();
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
    read_poly_list_file("test_triangulate_MONO.txt", &mut expected_result).unwrap();
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
    read_poly_list_file("test_convexpartition_HM.txt", &mut expected_result).unwrap();
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
    pp.convex_partition_opt(&testpolys.front().unwrap(), &mut result);
    read_poly_list_file("test_convexpartition_OPT.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("conv_opt.bmp", &result);

    if failures > 0 {
        Err(failures)
    } else {
        Ok(())
    }
}