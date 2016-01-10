use std::fs::*;
use std::io::BufReader;
use std::io::BufRead;

extern crate picross;
use picross::*;

/// Iterator yielding all increasing series from 0..n to 0..size
/// Ex: if size==3 and n==2, the iterator yield
/// ```
/// vec![0,1], vec[0,2], vec[1,2]
/// ```
/// This iterator should be created with gen_increasing_series
struct IncreasingSeriesGenerator {
    n: usize,
    size: usize,
    series: Vec<usize>,
    already_started: bool,
}

impl Iterator for IncreasingSeriesGenerator {
    type Item = Vec<usize>;
    fn next(&mut self) -> Option<Vec<usize>> {
        if !self.already_started {
            self.already_started = true;
            return Some(self.series.clone());
        }

        let mut i = self.n - 1;
        loop {
            if self.series[i] < (if i == self.n - 1 {self.size} else {self.series[i+1]}) - 1 {
                // on peut incrémenter cet indice safely
                break
            }
            if i == 0 {
                // itérateur terminé
                return None;
            }
            i -= 1;
        }
        self.series[i] += 1;
        for j in i+1..self.n {
            self.series[j] = self.series[j-1] + 1;
        }
        return Some(self.series.clone())
    }
}

/// Constructor for IncreasingSeriesGenerator
/// This function returns an iterator yielding all increasing series from 0..n to 0..size
/// ```
/// for series in gen_increasing_series(2, 3) {
///     // row will be successively vec![0,1], vec[0,2], vec[1,2]
/// }
/// ```
fn gen_increasing_series(n: usize, size: usize) -> IncreasingSeriesGenerator {
    IncreasingSeriesGenerator {
        n: n,
        size: size,
        series: (0..n).collect(),
        already_started: false,
    }
}

fn inc_series_to_row(series: &Vec<usize>, spec: &Vec<usize>, row_size: usize) -> Vec<Cell> {
    let mut row = Vec::with_capacity(row_size);
    let mut cur_pos = 0; // position dans series
    for (i_block, &pos) in series.iter().enumerate() {
        for _ in cur_pos..pos {
            row.push(Cell::White);
        }
        cur_pos = pos;
        for _ in 0..spec[i_block] {
            row.push(Cell::Black);
        }
        cur_pos += 1;
        if row.len() != row_size {
            row.push(Cell::White);
        }
    }
    for _ in row.len()..row_size {
        row.push(Cell::White);
    }
    row
}

/// Iterator yielding all possible picross rows following the given constraints
/// It should be created with gen_picross_row
struct PicrossRowGenerator<'a> {
    /// size of the row
    row_size : usize,
    /// specification of the blocks : &vec![1,2] means a one-cell block and a two-cell block
    spec : &'a Vec<usize>,
    inc_series_gen : IncreasingSeriesGenerator,
}

impl<'a> Iterator for PicrossRowGenerator<'a> {
    type Item = Vec<Cell>;

    fn next(&mut self) -> Option<Vec<Cell>> {
        match self.inc_series_gen.next() {
            Some(vec) => Some(inc_series_to_row(&vec, self.spec, self.row_size)),
            None => None
        }
    }
}

/// Returns an iterator yielding all possible picross rows following the given constraints :
/// row_size: size of the row
/// spec: specification of the blocks : &vec![1,2] means a one-cell block and a two-cell block
fn gen_picross_rows<'a>(row_size: usize, spec: &'a Vec<usize>) -> PicrossRowGenerator {
    PicrossRowGenerator {
        row_size: row_size,
        spec: spec,
        inc_series_gen: gen_increasing_series(spec.len(), row_size + 1 - spec.iter().fold(0, |sum, x| sum + x))
    }
}

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

fn gcd(start_row: &Vec<Cell>, mut possible_rows: PicrossRowGenerator) -> (Vec<Cell>, Vec<Vec<Cell>>) {
    let mut gcd = possible_rows.find(|row| is_row_consistent_with(start_row, row)).expect("No possibility...");
    let mut filtered_rows = vec!(gcd.clone());
    for row in possible_rows {
        if is_row_consistent_with(start_row, &row) {
            for pair in (&mut gcd).iter_mut().zip(row.iter()) {
                match pair {
                    (&mut Cell::Unknown, _) => (),
                    (mut known, new) => if new != known {*known = Cell::Unknown}
                }
            }
            filtered_rows.push(row);
        }
    }
    (gcd, filtered_rows)
}

fn backtrack_from(picross: &mut Picross, start_row: usize) -> bool {
    if start_row == picross.height {
        return true;
    }
    let (gcd_row, possible_rows) = gcd(&picross.cells[start_row], gen_picross_rows(picross.length, &picross.row_spec[start_row]) );
    for test_row in possible_rows {
        picross.cells[start_row] = test_row;
        if is_consistent(picross) {
            if backtrack_from(picross, start_row + 1) {
                return true;
            }
        }
    }
    picross.cells[start_row] = gcd_row;
    false
}

/// Fills picross with the first solution at finds.
/// If no solution is found, picross is left untouched.
/// Returns whether a solution has been found.
fn backtrack(picross: &mut Picross) -> bool {
    backtrack_from(picross, 0)
}

fn main() {
    for test_file in read_dir("../data").unwrap() {
        let f = File::open(test_file.unwrap().path()).unwrap();
        let mut picross = Picross::parse(&mut BufReader::new(f).lines().map(|x| x.unwrap()));
        // se forcer à voir un X
        if picross.length == 9 {
            picross.cells[4][4]=Cell::Black;
        }
        backtrack(&mut picross);
        println!("{}", picross.to_string());
        assert!(picross.is_valid())
    }
}
