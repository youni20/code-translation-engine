use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::collections::LinkedList;
use std::f64;

mod polypartition {
    use std::collections::LinkedList;
    use std::io;

    pub struct TPPLPoly;
    pub struct TPPLPoint { pub x: f64, pub y: f64 }
    pub struct Image;
    pub struct ImageIO;
    pub struct TPPLPartition;

    pub struct Pixel { pub r: u8, pub g: u8, pub b: u8 }

    impl TPPLPoly {
        pub fn init(&mut self, _numpoints: usize) {}
        pub fn set_hole(&mut self, _hole: bool) {}
        pub fn get_num_points(&self) -> usize { 0 }
        pub fn get_point(&self, _index: usize) -> TPPLPoint { TPPLPoint { x: 0.0, y: 0.0 } }
        pub fn is_hole(&self) -> bool { false }
    }

    impl Image {
        pub fn new(_width: u32, _height: u32) -> Self { Image }
        pub fn clear(&mut self, _color: Pixel) {}
        pub fn draw_line(&self, _x1: i32, _y1: i32, _x2: i32, _y2: i32, _color: &Pixel) {}
        pub fn width(&self) -> u32 { 500 }
        pub fn height(&self) -> u32 { 500 }
    }

    impl ImageIO {
        pub fn new() -> Self { ImageIO }
        pub fn save_image(&self, _filename: &str, _image: &Image) -> io::Result<()> { Ok(()) }
    }

    impl TPPLPartition {
        pub fn new() -> Self { TPPLPartition }
        pub fn triangulate_ec(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> io::Result<()> { Ok(()) }
        pub fn triangulate_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) -> io::Result<()> { Ok(()) }
        pub fn triangulate_mono(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> io::Result<()> { Ok(()) }
        pub fn convex_partition_hm(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> io::Result<()> { Ok(()) }
        pub fn convex_partition_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) -> io::Result<()> { Ok(()) }
    }
}

use polypartition::*;

fn read_poly_from_file(file: &mut BufReader<File>, poly: &mut TPPLPoly) -> io::Result<()> {
    let mut line = String::new();
    file.read_line(&mut line)?;
    let numpoints: usize = line.trim().parse().unwrap();
    poly.init(numpoints);

    line.clear();
    file.read_line(&mut line)?;
    let hole: i32 = line.trim().parse().unwrap();
    if hole != 0 {
        poly.set_hole(true);
    }

    for _ in 0..numpoints {
        line.clear();
        file.read_line(&mut line)?;
        let coords: Vec<f64> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        poly.init(coords.len());
    }
    Ok(())
}

fn read_poly(filename: &str, poly: &mut TPPLPoly) -> io::Result<()> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    read_poly_from_file(&mut reader, poly)
}

fn read_poly_list_from_file(file: &mut BufReader<File>, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    polys.clear();
    let mut line = String::new();
    file.read_line(&mut line)?;
    let numpolys: usize = line.trim().parse().unwrap();

    for _ in 0..numpolys {
        let mut poly = TPPLPoly;
        read_poly_from_file(file, &mut poly)?;
        polys.push_back(poly);
    }

    Ok(())
}

fn read_poly_list(filename: &str, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    read_poly_list_from_file(&mut reader, polys)
}

fn write_poly_to_file(file: &mut File, poly: &TPPLPoly) -> io::Result<()> {
    writeln!(file, "{}", poly.get_num_points())?;

    if poly.is_hole() {
        writeln!(file, "1")?;
    } else {
        writeln!(file, "0")?;
    }

    for i in 0..poly.get_num_points() {
        writeln!(file, "{} {}", poly.get_point(i).x, poly.get_point(i).y)?;
    }

    Ok(())
}

fn write_poly(filename: &str, poly: &TPPLPoly) -> io::Result<()> {
    let mut file = File::create(filename)?;
    write_poly_to_file(&mut file, poly)
}

fn write_poly_list_to_file(file: &mut File, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    writeln!(file, "{}", polys.len())?;
    for poly in polys {
        write_poly_to_file(file, poly)?;
    }
    Ok(())
}

fn write_poly_list(filename: &str, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut file = File::create(filename)?;
    write_poly_list_to_file(&mut file, polys)
}

fn draw_poly(img: &mut Image, poly: &TPPLPoly, xmin: f64, xmax: f64, ymin: f64, ymax: f64) {
    let color = Pixel { r: 0, g: 0, b: 0 };

    let imgmin = TPPLPoint { x: 5.0, y: 5.0 };
    let polymin = TPPLPoint { x: xmin, y: ymin };

    let poly_size_x = xmax - xmin;
    let poly_size_y = ymax - ymin;
    let img_size_x = img.width() as f64 - 10.0;
    let img_size_y = img.height() as f64 - 10.0;

    let mut scale = 1.0;
    if poly_size_x > 0.0 {
        scale = img_size_x / poly_size_x;
    }
    if poly_size_y > 0.0 {
        scale = f64::min(scale, img_size_y / poly_size_y);
    }

    for i in 0..poly.get_num_points() {
        let p1 = poly.get_point(i);
        let p2 = poly.get_point((i + 1) % poly.get_num_points());
        let p1img = TPPLPoint {x: (p1.x - polymin.x) * scale + imgmin.x, y: (p1.y - polymin.y) * scale + imgmin.y};
        let p2img = TPPLPoint {x: (p2.x - polymin.x) * scale + imgmin.x, y: (p2.y - polymin.y) * scale + imgmin.y};
        img.draw_line(p1img.x as i32, p1img.y as i32, p2img.x as i32, p2img.y as i32, &color);
    }
}

fn draw_poly_file(filename: &str, poly: &TPPLPoly) -> io::Result<()> {
    let mut img = Image::new(500, 500);
    img.clear(Pixel { r: 255, g: 255, b: 255 });

    let (mut xmin, mut xmax) = (f64::MAX, f64::MIN);
    let (mut ymin, mut ymax) = (f64::MAX, f64::MIN);

    for i in 0..poly.get_num_points() {
        xmin = xmin.min(poly.get_point(i).x);
        xmax = xmax.max(poly.get_point(i).x);
        ymin = ymin.min(poly.get_point(i).y);
        ymax = ymax.max(poly.get_point(i).y);
    }

    draw_poly(&mut img, poly, xmin, xmax, ymin, ymax);

    let io = ImageIO::new();
    io.save_image(filename, &img)
}

fn draw_poly_list(filename: &str, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut img = Image::new(300, 450);
    img.clear(Pixel { r: 255, g: 255, b: 255 });

    let (mut xmin, mut xmax) = (f64::MAX, f64::MIN);
    let (mut ymin, mut ymax) = (f64::MAX, f64::MIN);

    for poly in polys {
        for i in 0..poly.get_num_points() {
            xmin = xmin.min(poly.get_point(i).x);
            xmax = xmax.max(poly.get_point(i).x);
            ymin = ymin.min(poly.get_point(i).y);
            ymax = ymax.max(poly.get_point(i).y);
        }
    }

    for poly in polys {
        draw_poly(&mut img, poly, xmin, xmax, ymin, ymax);
    }

    let io = ImageIO::new();
    io.save_image(filename, &img)
}

fn compare_poly(p1: &TPPLPoly, p2: &TPPLPoly) -> bool {
    p1.get_num_points() == p2.get_num_points()
}

fn compare_poly_list(polys1: &LinkedList<TPPLPoly>, polys2: &LinkedList<TPPLPoly>) -> bool {
    if polys1.len() != polys2.len() {
        return false;
    }

    let iter1 = polys1.iter();
    let iter2 = polys2.iter();
    for (p1, p2) in iter1.zip(iter2) {
        if !compare_poly(p1, p2) {
            return false;
        }
    }

    true
}

fn generate_test_data() {
    let pp = TPPLPartition::new();

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list("test_input.txt", &mut testpolys).unwrap();

    draw_poly_list("test_input.bmp", &testpolys).unwrap();

    pp.triangulate_ec(&testpolys, &mut result).unwrap();
    draw_poly_list("test_triangulate_EC.bmp", &result).unwrap();
    write_poly_list("test_triangulate_EC.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.triangulate_opt(&testpolys.front().unwrap(), &mut result).unwrap();
    draw_poly_list("test_triangulate_OPT.bmp", &result).unwrap();
    write_poly_list("test_triangulate_OPT.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.triangulate_mono(&testpolys, &mut result).unwrap();
    draw_poly_list("test_triangulate_MONO.bmp", &result).unwrap();
    write_poly_list("test_triangulate_MONO.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.convex_partition_hm(&testpolys, &mut result).unwrap();
    draw_poly_list("test_convexpartition_HM.bmp", &result).unwrap();
    write_poly_list("test_convexpartition_HM.txt", &result).unwrap();

    result.clear();
    expected_result.clear();

    pp.convex_partition_opt(&testpolys.front().unwrap(), &mut result).unwrap();
    draw_poly_list("test_convexpartition_OPT.bmp", &result).unwrap();
    write_poly_list("test_convexpartition_OPT.txt", &result).unwrap();
}

fn main() -> Result<(), i32> {
    let mut failures = 0;

    let pp = TPPLPartition::new();

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result: LinkedList<TPPLPoly> = LinkedList::new();

    read_poly_list("test_input.txt", &mut testpolys).unwrap();

    draw_poly_list("test_input.bmp", &testpolys).unwrap();

    println!("Testing Triangulate_EC: ");
    pp.triangulate_ec(&testpolys, &mut result).unwrap();
    read_poly_list("test_triangulate_EC.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("tri_ec.bmp", &result).unwrap();

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_OPT: ");
    pp.triangulate_opt(&testpolys.front().unwrap(), &mut result).unwrap();
    read_poly_list("test_triangulate_OPT.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("tri_opt.bmp", &result).unwrap();

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_MONO: ");
    pp.triangulate_mono(&testpolys, &mut result).unwrap();
    read_poly_list("test_triangulate_MONO.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("tri_mono.bmp", &result).unwrap();

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_HM: ");
    pp.convex_partition_hm(&testpolys, &mut result).unwrap();
    read_poly_list("test_convexpartition_HM.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("conv_hm.bmp", &result).unwrap();

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_OPT: ");
    pp.convex_partition_opt(&testpolys.front().unwrap(), &mut result).unwrap();
    read_poly_list("test_convexpartition_OPT.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list("conv_opt.bmp", &result).unwrap();

    if failures == 0 {
        Ok(())
    } else {
        Err(failures)
    }
}