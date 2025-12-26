#!/bin/bash
cd source
cd compiler
cargo build --release
cd ..
cd plsa
g++ main.cpp -o plsa
cd ..
cd updater
go get updater
go build
cd ..
cd diagnostic
cargo build --release
cd ..
cd preprocessor
gcc main.c -o preprocessor 
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
cd ..
cd virus
go get virus
go build
