#![feature(iter_arith)]

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
        inc_series_gen: gen_increasing_series(spec.len(), row_size + 1 - spec.iter().sum::<usize>())
    }
}

fn main() {
    for row in gen_picross_rows(7, &vec![1, 2]) {
        println!("{:?}", row);
    }
}
