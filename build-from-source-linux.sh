#!/bin/bash
# windows -> x86_64-w64-mingw32-g++ main.cpp -o program.exe
cd source
cd plsa
g++ main.cpp -o plsa
cd ..
cd translator
cargo build --release
cd ..
cd compiler
cargo build --release
cd ..
cd interpreter 
cargo build --release
cd ..
cd vm
cargo build --release
cd ..
cd updater
go get updater
go build 
cd ..
cd ..
cd cli
cd vira
go get vira
go build
cd ..
cd virac
go get virac
go build
