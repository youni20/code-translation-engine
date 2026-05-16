use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::f32::{MAX, MIN};
use std::collections::LinkedList;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TPPLPoint {
    x: f32,
    y: f32,
}

type TPPLFloat = f32;  // Define TPPLFloat as f32

#[derive(Clone, Debug)]
struct TPPLPoly {
    points: Vec<TPPLPoint>,
    hole: bool,
}

impl TPPLPoly {
    fn init(&mut self, numpoints: usize) {
        self.points = vec![TPPLPoint { x: 0.0, y: 0.0 }; numpoints];
    }
    
    fn set_hole(&mut self, hole: bool) {
        self.hole = hole;
    }
    
    fn get_num_points(&self) -> usize {
        self.points.len()
    }
    
    fn get_point(&self, i: usize) -> TPPLPoint {
        self.points[i]
    }
}

fn read_poly(fp: &mut File, poly: &mut TPPLPoly) -> io::Result<()> {
    let mut reader = BufReader::new(fp);
    let mut numpoints = String::new();
    reader.read_line(&mut numpoints)?;
    let numpoints: usize = numpoints.trim().parse().unwrap();
    poly.init(numpoints);

    let mut hole = String::new();
    reader.read_line(&mut hole)?;
    let hole: i32 = hole.trim().parse().unwrap();
    if hole != 0 {
        poly.set_hole(true);
    }

    for i in 0..numpoints {
        let mut coords = String::new();
        reader.read_line(&mut coords)?;
        let mut coords_iter = coords.split_whitespace();
        let x: f32 = coords_iter.next().unwrap().parse().unwrap();
        let y: f32 = coords_iter.next().unwrap().parse().unwrap();
        poly.points[i] = TPPLPoint { x, y };
    }

    Ok(())
}

fn read_poly_file(filename: &str, poly: &mut TPPLPoly) -> io::Result<()> {
    let mut fp = File::open(filename)?;
    read_poly(&mut fp, poly)?;
    Ok(())
}

fn read_poly_list(fp: &mut File, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut reader = BufReader::new(&mut *fp);
    polys.clear();
    let mut numpolys = String::new();
    reader.read_line(&mut numpolys)?;
    let numpolys: usize = numpolys.trim().parse().unwrap();
    for _ in 0..numpolys {
        let mut poly = TPPLPoly { points: Vec::new(), hole: false };
        read_poly(fp, &mut poly)?;
        polys.push_back(poly);
    }

    Ok(())
}

fn read_poly_list_file(filename: &str, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut fp = File::open(filename)?;
    read_poly_list(&mut fp, polys)?;
    Ok(())
}

fn write_poly(fp: &mut File, poly: &TPPLPoly) -> io::Result<()> {
    writeln!(fp, "{}", poly.get_num_points())?;
    writeln!(fp, "{}", if poly.hole { 1 } else { 0 })?;

    for point in &poly.points {
        writeln!(fp, "{} {}", point.x, point.y)?;
    }

    Ok(())
}

fn write_poly_file(filename: &str, poly: &TPPLPoly) -> io::Result<()> {
    let mut fp = File::create(filename)?;
    write_poly(&mut fp, poly)?;
    Ok(())
}

fn write_poly_list(fp: &mut File, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    writeln!(fp, "{}", polys.len())?;

    for poly in polys {
        write_poly(fp, poly)?;
    }

    Ok(())
}

fn write_poly_list_file(filename: &str, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut fp = File::create(filename)?;
    write_poly_list(&mut fp, polys)?;
    Ok(())
}

struct Image {
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl Image {
    fn new(width: usize, height: usize) -> Image {
        Image { width, height }
    }

    fn clear(&mut self, _color: Pixel) {}

    fn draw_line(&mut self, _x1: i32, _y1: i32, _x2: i32, _y2: i32, _color: Pixel) {}
}

fn draw_poly(img: &mut Image, poly: &TPPLPoly, xmin: TPPLFloat, xmax: TPPLFloat, ymin: TPPLFloat, ymax: TPPLFloat) {
    #[derive(Clone, Debug)]
    struct Point {
        x: TPPLFloat,
        y: TPPLFloat,
    }
    
    let poly_min = Point { x: xmin, y: ymin };
    let img_min = Point { x: 5.0, y: 5.0 };

    let poly_size_x = xmax - xmin;
    let poly_size_y = ymax - ymin;
    let img_size_x = (img.width as TPPLFloat) - 10.0;
    let img_size_y = (img.height as TPPLFloat) - 10.0;

    let mut scalex = 0.0;
    let mut scaley = 0.0;
    let scale;

    if poly_size_x > 0.0 {
        scalex = img_size_x / poly_size_x;
    }
    if poly_size_y > 0.0 {
        scaley = img_size_y / poly_size_y;
    }

    if scalex > 0.0 && scalex < scaley {
        scale = scalex;
    } else if scaley > 0.0 {
        scale = scaley;
    } else {
        scale = 1.0;
    }

    let color = Pixel { r: 0, g: 0, b: 0 };

    for i in 0..poly.get_num_points() {
        let p1 = poly.get_point(i);
        let p2 = poly.get_point((i + 1) % poly.get_num_points());

        let p1img = Point { x: (p1.x - poly_min.x) * scale + img_min.x, y: (p1.y - poly_min.y) * scale + img_min.y };
        let p2img = Point { x: (p2.x - poly_min.x) * scale + img_min.x, y: (p2.y - poly_min.y) * scale + img_min.y };

        img.draw_line(p1img.x as i32, p1img.y as i32, p2img.x as i32, p2img.y as i32, color);
    }
}

struct ImageIO;

impl ImageIO {
    fn save_image(&self, _filename: &str, _img: &Image) -> Result<(), io::Error> {
        Ok(())
    }
}

fn draw_poly_file(filename: &str, poly: &TPPLPoly) {
    let mut img = Image::new(500, 500);
    let white = Pixel { r: 255, g: 255, b: 255 };
    img.clear(white);
    let io = ImageIO;

    let mut xmin = MAX;
    let mut xmax = MIN;
    let mut ymin = MAX;
    let mut ymax = MIN;

    for i in 0..poly.get_num_points() {
        let point = poly.get_point(i);
        if point.x < xmin { xmin = point.x; }
        if point.x > xmax { xmax = point.x; }
        if point.y < ymin { ymin = point.y; }
        if point.y > ymax { ymax = point.y; }
    }

    draw_poly(&mut img, poly, xmin, xmax, ymin, ymax);
    io.save_image(filename, &img).unwrap();
}

fn draw_poly_list_file(filename: &str, polys: &LinkedList<TPPLPoly>) {
    let mut img = Image::new(300, 450);
    let white = Pixel { r: 255, g: 255, b: 255 };
    img.clear(white);
    let io = ImageIO;

    let mut xmin = MAX;
    let mut xmax = MIN;
    let mut ymin = MAX;
    let mut ymax = MIN;

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

    io.save_image(filename, &img).unwrap();
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

fn compare_poly_list(polys1: &LinkedList<TPPLPoly>, polys2: &LinkedList<TPPLPoly>) -> bool {
    if polys1.len() != polys2.len() {
        return false;
    }

    let mut iter1 = polys1.iter();
    let mut iter2 = polys2.iter();

    for _ in 0..polys1.len() {
        if !compare_poly(iter1.next().unwrap(), iter2.next().unwrap()) {
            return false;
        }
    }

    true
}

struct TPPLPartition;

impl TPPLPartition {
    fn triangulate_ec(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> bool {
        true
    }

    fn triangulate_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) -> bool {
        true
    }

    fn triangulate_mono(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> bool {
        true
    }

    fn convex_partition_hm(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> bool {
        true
    }

    fn convex_partition_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) -> bool {
        true
    }
}

fn main() {
    let mut failures = 0;
    let pp = TPPLPartition;

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result = LinkedList::new();

    read_poly_list_file("test_input.txt", &mut testpolys).unwrap();

    draw_poly_list_file("test_input.bmp", &testpolys);

    println!("Testing Triangulate_EC: ");
    pp.triangulate_ec(&testpolys, &mut result);
    read_poly_list_file("test_triangulate_EC.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }
    draw_poly_list_file("tri_ec.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_OPT: ");
    if let Some(first_poly) = testpolys.front() {
        pp.triangulate_opt(first_poly, &mut result);
        read_poly_list_file("test_triangulate_OPT.txt", &mut expected_result).unwrap();
        if compare_poly_list(&result, &expected_result) {
            println!("success");
        } else {
            println!("failed");
            failures += 1;
        }
        draw_poly_list_file("tri_opt.bmp", &result);
    }

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
    draw_poly_list_file("tri_mono.bmp", &result);

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
    draw_poly_list_file("conv_hm.bmp", &result);

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_OPT: ");
    if let Some(first_poly) = testpolys.front() {
        pp.convex_partition_opt(first_poly, &mut result);
        read_poly_list_file("test_convexpartition_OPT.txt", &mut expected_result).unwrap();
        if compare_poly_list(&result, &expected_result) {
            println!("success");
        } else {
            println!("failed");
            failures += 1;
        }
        draw_poly_list_file("conv_opt.bmp", &result);
    }

    std::process::exit(failures);
}