# picrossbacktrack

Solves picross problems with a simplistic but nice gui.

## Installation
on debian-like, run
```
apt install libcsfml-dev cargo-stable
```
Checkout the repository, and inside the directory, 
```
cargo build --release
```
The binary is then available at the location:
```
target/release/picrossbacktrack
```

## Usage
```
Usage:
    target/release/picrossbacktrack [OPTIONS] FILE

Solves a picross grid.

positional arguments:
  file                  File to solve

optional arguments:
  -h,--help             show this help message and exit
  -s,--sync             Display picross solving synchonously
```

## File format
The first two lines give respectively the number of rows and the number of columns. Then, each line gives the clue for each row
(from top to bottom) and for each column (from left to right). For instance :
```
9
9
[3,3]
[1,1]
etc.
```

## Details

You can generate documentation by running 
```
cargo doc
```
It will be located in ```target/doc```
