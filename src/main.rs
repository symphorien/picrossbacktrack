use std::cmp;
use std::env;
use std::fs::*;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;

extern crate argparse;
use argparse::{ArgumentParser, Store, StoreTrue};

extern crate picross;
use picross::*;

extern crate sfml;
use sfml::system::Vector2f;
use sfml::window::{ContextSettings, VideoMode, event, Close};
use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, Color};

fn is_consistent(picross: &Picross) -> bool {
    for col in 0..picross.length {
        let mut num_block = 0;
        let mut size_block = 0;
        let mut dirty = false; // whether there is an unknown cell in this column
        for row in 0..picross.height {
            match picross.cells[row][col] {
                Cell::Unknown => {dirty = true; break},
                Cell::Black   => size_block += 1,
                Cell::White   => {
                    if size_block > 0 {
                        if num_block >= picross.col_spec[col].len() || size_block != picross.col_spec[col][num_block] {
                            return false;
                        }
                        num_block += 1;
                        size_block = 0;
                    }
                }
            }
        }
        if dirty {
            // we stumbled upon an unknown cell, just check that the picross could be filled up further
            if size_block > 0 {
                if num_block >= picross.col_spec[col].len() || size_block > picross.col_spec[col][num_block] {
                    return false;
                }
                num_block += 1;
            }
            if num_block > picross.col_spec[col].len() {
                return false;
            }
        } else {
            // we got to the end of the column, check the last block has exactly the good size
            if size_block > 0 {
                if num_block >= picross.col_spec[col].len() || size_block != picross.col_spec[col][num_block] {
                    return false;
                }
                num_block += 1;
            }
            if num_block != picross.col_spec[col].len() {
                return false;
            }
        };
    };
    true
}

/// Checks whether new is a picross row filling old, which means no known cell changes value.
fn is_row_consistent_with(old: &Vec<Cell>, new: &Vec<Cell>) -> bool {
    old.iter().zip(new.iter()).all(|pair|
        match pair {
            (&Cell::Unknown, _) => true,
            (_, &Cell::Unknown) => false,
            (old_known, new_known) => old_known == new_known
        })
}

fn gcd<'a, T> (start_row: &Vec<Cell>, mut possible_rows: T) -> (Vec<Cell>, bool) where T: Iterator<Item=&'a Vec<Cell>> {
    let mut gcd = possible_rows.find(|row| is_row_consistent_with(start_row, row)).expect("No solution to this picross").clone();
    for row in possible_rows {
        if is_row_consistent_with(start_row, &row) {
            for pair in (&mut gcd).iter_mut().zip(row.iter()) {
                match pair {
                    (&mut Cell::Unknown, _) => (),
                    (mut known, new) => if new != known {*known = Cell::Unknown}
                }
            }
        }
    }
    let dirty = start_row.iter().zip(gcd.iter()).any(|(x, y)| x != y);
    (gcd, dirty)
}

fn combex_rows(picross: &mut Picross, w: &mut RenderWindow) -> bool {
    let mut dirty = false;
    for (row_num, row) in (0..picross.cells.len()).enumerate() {
        let res = gcd(&picross.cells[row], picross.possible_rows[row_num].iter());
        picross.cells[row] = res.0;
        dirty |= res.1;
        if res.1 {
            draw(w, &*picross);
        }
    }
    dirty
}

fn combex_cols(picross: &mut Picross, w: &mut RenderWindow) -> bool {
    let mut dirty = false;
    for (col_num, col) in picross.transpose().iter().enumerate() {
        let res = gcd(&col, picross.possible_cols[col_num].iter());
        picross.set_col(col_num, res.0);
        dirty |= res.1;
        if res.1 {
            draw(w, &picross);
        }
    }
    dirty
}

fn backtrack_from(picross: &mut Picross, start_row: usize, w: &mut RenderWindow) -> bool {
    if start_row == picross.height {
        return true;
    }
    let original_row = picross.cells[start_row].clone();
    let unknown_original_row = original_row.iter().all(|x| x == &Cell::Unknown);
    let known_original_row = original_row.iter().all(|x| x != &Cell::Unknown);
    draw(w, &picross); // Do not draw *every* backtracking, to save time due to vertical sync
    // To draw every step, just move this line into the for loop
    if known_original_row {
        if backtrack_from(picross, start_row + 1, w) {
            return true;
        }
    } else {
        for test_row in picross.possible_rows[start_row].clone().iter() {
            if unknown_original_row || is_row_consistent_with(&original_row, &test_row) {
                picross.cells[start_row] = test_row.clone();
                if is_consistent(picross) {
                    if backtrack_from(picross, start_row + 1, w) {
                        return true;
                    }
                }
            }
        }
    }
    picross.cells[start_row] = original_row;
    false
}

/// Fills picross with the first solution it finds.
/// If no solution is found, picross is left untouched.
/// Returns whether a solution has been found.
fn backtrack(picross: &mut Picross, w: &mut RenderWindow) -> bool {
    while combex_rows(picross, w) | combex_cols(picross, w) {
        draw(w, &picross);
    }
    backtrack_from(picross, 0, w);
    draw(w, &picross);
    true
}

/// Draws `Picross` `p` to `RenderWindow` `w`
/// Assumes `w` is 600x600
fn draw(w: &mut RenderWindow, p: &Picross) {
    if match env::var("sync") {
        Ok(val) =>val.len()>0,
        Err(_) => false
    } {
        w.clear(&Color::new_rgb(127, 127, 127));

        let sq_side = 600. / (cmp::max(p.height, p.length) as f32);

        let mut sq = match RectangleShape::new() {
            Some(sq) => sq,
            None     => panic!("Error, cannot create square")
        };
        sq.set_size(&Vector2f::new(sq_side - 2., sq_side - 2.));

        for y in 0..p.height {
            for x in 0..p.length {
                sq.set_position(&Vector2f::new((x as f32) * sq_side + 1., (y as f32) * sq_side + 1.));
                sq.set_fill_color(&match p.cells[y][x] {
                    Cell::Black => Color::black(),
                    Cell::White => Color::white(),
                    Cell::Unknown => Color::new_rgb(128, 128, 128)
                });
                w.draw(&sq);
            }
        }

        w.display();

        for event in w.events() {
            match event {
                event::Closed => { panic!("Interrupted") },
                _             => { /* ignore */ }
            }
        }
    }
}

fn main() {
    let mut file = "".to_owned();
    let mut sync = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Solves a picross grid.");
        ap.refer(&mut sync)
            .add_option(&["-s", "--sync"], StoreTrue,
                        "Display picross solving synchonously");
        ap.refer(&mut file).required()
            .add_argument("file", Store,
                          "File to solve");
        ap.parse_args_or_exit();
    }
    env::set_var("sync", if sync {"1"} else {""});
    let mut window = match RenderWindow::new(VideoMode::new_init(600, 600, 32),
                                             "Picross",
                                             Close,
                                             &ContextSettings::default()) {
        Some(window) => window,
        None => panic!("Cannot create a new Render Window.")
    };

    let f = File::open(Path::new(&file)).expect(&format!("Could not open {}", file));
    let mut picross = Picross::parse(&mut BufReader::new(f).lines().map(|x| x.expect("Read error")));
    assert_eq!(picross.length, picross.cells[0].len());
    assert_eq!(picross.height, picross.cells.len());
    picross.fill_possibles();
    backtrack(&mut picross, &mut window);
    println!("{}", picross.to_string());
    assert!(picross.is_valid());
    env::set_var("sync", "1");
    draw(&mut window, &mut picross);
    println!("Press any key to quit");
    let mut waiting = true;
    while waiting {
        for event in window.events() {
            match event {
                event::Closed => panic!("Interrupted"),
                event::KeyReleased { code: _, alt: _, ctrl: _, shift: _, system: _ } => waiting = false,
                _ => { /* ignore */ }
            }
        }
    }
}
