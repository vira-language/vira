# build.ps1

# source/compiler
Set-Location source/compiler
cargo build --release

# source/plsa
Set-Location ..
Set-Location plsa
g++ main.cpp -o plsa

# source/updater
Set-Location ..
Set-Location updater
go get updater
go build

# source/diagnostic
Set-Location ..
Set-Location diagnostic
cargo build --release

# source/preprocessor
Set-Location ..
Set-Location preprocessor
gcc main.c -o preprocessor

# powrót do root
Set-Location ..
Set-Location ..

# cli/vira
Set-Location cli/vira
go get vira
go build

# cli/virac
Set-Location ..
Set-Location virac
go get virac
go build

# cli/virus
Set-Location ..
Set-Location virus
go get virus
go build

