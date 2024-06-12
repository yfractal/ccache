# Go calls C

gcc -c -o example.o example.c
ar rcs libexample.a example.o

go run main.go

# Go -> C -> Go
gcc -c -o example.o example.c
ar rcs libexample.a example.o

go run main.go serisizor.go

# Go X Rust
## Rust calls dynamic
gcc -c -o example.o example.c
ar rcs libexample.a example.o
cargo run --bin main

## Go -> Rust -> Go
cargo build --release // CallEncode doesn't
go run main.go serisizor.go // go compiled CallEncode in serisizor