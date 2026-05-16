use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::collections::LinkedList;

mod polypartition {
    use std::collections::LinkedList;

    #[derive(Clone, PartialEq)]
    pub struct TPPLPoint {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Clone)]
    pub struct TPPLPoly {
        pub points: Vec<TPPLPoint>,
        pub hole: bool,
    }

    impl TPPLPoly {
        pub fn new() -> TPPLPoly {
            TPPLPoly {
                points: Vec::new(),
                hole: false,
            }
        }
        
        pub fn init(&mut self, numpoints: usize) {
            self.points = vec![TPPLPoint { x: 0.0, y: 0.0 }; numpoints];
        }

        pub fn set_hole(&mut self, hole: bool) {
            self.hole = hole;
        }

        pub fn get_num_points(&self) -> usize {
            self.points.len()
        }

        pub fn get_point(&self, index: usize) -> &TPPLPoint {
            &self.points[index]
        }
    }
    
    pub struct TPPLPartition;

    impl TPPLPartition {
        pub fn triangulate_ec(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> bool {
            true
        }

        pub fn triangulate_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) -> bool {
            true
        }

        pub fn triangulate_mono(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> bool {
            true
        }

        pub fn convex_partition_hm(&self, _polys: &LinkedList<TPPLPoly>, _result: &mut LinkedList<TPPLPoly>) -> bool {
            true
        }

        pub fn convex_partition_opt(&self, _poly: &TPPLPoly, _result: &mut LinkedList<TPPLPoly>) -> bool {
            true
        }
    }
}

use polypartition::TPPLPoly;
use polypartition::TPPLPartition;

fn read_poly(file: &mut File, poly: &mut TPPLPoly) -> io::Result<()> {
    let mut numpoints = 0;
    let mut hole = 0;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let mut lines = buf.lines();

    if let Some(line) = lines.next() {
        numpoints = line.parse().unwrap_or(0);
    }
    poly.init(numpoints);

    if let Some(line) = lines.next() {
        hole = line.parse().unwrap_or(0);
    }
    if hole != 0 {
        poly.set_hole(true);
    }

    for i in 0..numpoints {
        if let Some(line) = lines.next() {
            let mut coords = line.split_whitespace();
            if let (Some(x), Some(y)) = (coords.next(), coords.next()) {
                poly.points[i].x = x.parse().unwrap();
                poly.points[i].y = y.parse().unwrap();
            }
        }
    }

    Ok(())
}

fn read_poly_list<P: AsRef<Path>>(path: P, polys: &mut LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let mut numpolys = 0;
    let mut lines = buf.lines();

    if let Some(line) = lines.next() {
        numpolys = line.parse().unwrap_or(0);
    }

    polys.clear();
    for _ in 0..numpolys {
        let mut poly = TPPLPoly::new();
        read_poly(&mut file, &mut poly)?;
        polys.push_back(poly);
    }

    Ok(())
}

fn write_poly(file: &mut File, poly: &TPPLPoly) -> io::Result<()> {
    writeln!(file, "{}", poly.get_num_points())?;
    writeln!(file, "{}", if poly.hole { 1 } else { 0 })?;
    for point in &poly.points {
        writeln!(file, "{} {}", point.x, point.y)?;
    }
    Ok(())
}

fn write_poly_list<P: AsRef<Path>>(path: P, polys: &LinkedList<TPPLPoly>) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "{}", polys.len())?;
    for poly in polys {
        write_poly(&mut file, poly)?;
    }
    Ok(())
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
    for (p1, p2) in polys1.iter().zip(polys2) {
        if !compare_poly(p1, p2) {
            return false;
        }
    }
    true
}

fn main() -> Result<(), i32> {
    let mut failures = 0;
    let pp = TPPLPartition;

    let mut testpolys = LinkedList::new();
    let mut result = LinkedList::new();
    let mut expected_result = LinkedList::new();

    if read_poly_list("test_input.txt", &mut testpolys).is_err() {
        return Err(1);
    }

    println!("Testing Triangulate_EC: ");
    pp.triangulate_ec(&testpolys, &mut result);
    read_poly_list("test_triangulate_EC.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_OPT: ");
    if let Some(poly) = testpolys.front() {
        pp.triangulate_opt(poly, &mut result);
    }
    read_poly_list("test_triangulate_OPT.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }

    result.clear();
    expected_result.clear();

    println!("Testing Triangulate_MONO: ");
    pp.triangulate_mono(&testpolys, &mut result);
    read_poly_list("test_triangulate_MONO.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_HM: ");
    pp.convex_partition_hm(&testpolys, &mut result);
    read_poly_list("test_convexpartition_HM.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }

    result.clear();
    expected_result.clear();

    println!("Testing ConvexPartition_OPT: ");
    if let Some(poly) = testpolys.front() {
        pp.convex_partition_opt(poly, &mut result);
    }
    read_poly_list("test_convexpartition_OPT.txt", &mut expected_result).unwrap();
    if compare_poly_list(&result, &expected_result) {
        println!("success");
    } else {
        println!("failed");
        failures += 1;
    }

    if failures > 0 {
        Err(failures)
    } else {
        Ok(())
    }
}