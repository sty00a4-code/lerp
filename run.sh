clear
export RUST_BACKTRACE=1
cargo run -q -- test/test.lerp test/test.asm
nasm -f elf test/test.asm -o test/test.o
gcc -m32 test/test.o -no-pie -o test/test -O3
./test/test