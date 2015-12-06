#[derive(Clone, PartialEq, Debug)]
pub enum Cell {
    Unknown,
    Black,
    White,
} 

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

        

fn main() {
    let generator = gen_increasing_series(2, 5);
    for series in generator  {
        println!("{:?}", series);
        println!("{:?}", inc_series_to_row(&series, &vec![1,2], 7));
    }
}
