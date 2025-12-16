package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"

	"github.com/pterm/pterm"
)

var systemBinDir string
var libDir string
var binDir string
var exe string

func init() {
	if runtime.GOOS == "windows" {
		exe = ".exe"
		systemBinDir = `C:\\Program Files\\Vira\\bin`
		libDir = `C:\\Program Files\\Vira\\lib`
		binDir = libDir + `\\bin`
	} else {
		exe = ""
		systemBinDir = "/usr/bin"
		libDir = "/usr/lib/vira-lang"
		binDir = libDir + "/bin"
	}
}

func main() {
	if len(os.Args) < 2 {
		pterm.DefaultHeader.WithFullWidth().Println("Virac - Vira Compiler CLI")
		pterm.Info.Println("Usage:")
		pterm.Info.Println("  virac compile <file.vira> -o <output> [--target <windows|linux>] [--arch <x64>] [--bytecode]")
		pterm.Info.Println("  --target: windows or linux (default: current os)")
		pterm.Info.Println("  --arch: x64 (default)")
		pterm.Info.Println("  --bytecode: Compile to bytecode (.object) instead of native binary")
		return
	}

	cmd := os.Args[1]
	if cmd != "compile" {
		pterm.Error.Println("Unknown command. Use 'compile'")
		return
	}

	var output string
	var targetOS string
	var arch string
	var bytecode bool

	fs := flag.NewFlagSet("compile", flag.ExitOnError)
	fs.StringVar(&output, "o", "", "Output file")
	fs.StringVar(&targetOS, "target", runtime.GOOS, "Target OS: windows or linux")
	fs.StringVar(&arch, "arch", "x64", "Target architecture: x64")
	fs.BoolVar(&bytecode, "bytecode", false, "Compile to bytecode")
	fs.Parse(os.Args[2:])

	if len(fs.Args()) < 1 {
		pterm.Error.Println("Missing input file")
		return
	}
	input := fs.Args()[0]

	if output == "" {
		base := filepath.Base(input)
		ext := filepath.Ext(base)
		output = base[:len(base)-len(ext)]
		if bytecode {
			output += ".object"
		} else if targetOS == "windows" {
			output += ".exe"
		}
	}

	compile(input, output, targetOS, arch, bytecode)
}

func compile(input, output, targetOS, arch string, bytecode bool) {
	spinner, _ := pterm.DefaultSpinner.Start("Compiling...")

	var err error
	if bytecode {
		compilerBin := binDir + "/compiler" + exe
		cmd := exec.Command(compilerBin, input, "-o", output)
		err = cmd.Run()
	} else {
		// Translate to C
		tempC := filepath.Join(os.TempDir(), "vira_temp.c")
		translatorBin := binDir + "/translator" + exe
		transCmd := exec.Command(translatorBin, "translate", input, "--target", "c", "--output", tempC)
		err = transCmd.Run()
		if err != nil {
			spinner.Fail("Translation failed")
			pterm.Error.Printf("Error: %v\n", err)
			return
		}

		// Compile with zig
		var zigTarget string
		if targetOS == "windows" && arch == "x64" {
			zigTarget = "x86_64-windows-gnu"
		} else if targetOS == "linux" && arch == "x64" {
			zigTarget = "x86_64-linux-gnu"
		} else {
			spinner.Fail("Unsupported target")
			return
		}

		zigCmd := exec.Command("zig", "cc", "-target", zigTarget, "-o", output, tempC)
		err = zigCmd.Run()
		os.Remove(tempC)
	}

	if err != nil {
		spinner.Fail("Compilation failed")
		pterm.Error.Printf("Error: %v\n", err)
		return
	}

	spinner.Success("Compilation completed")
	pterm.Success.Printf("Output: %s\n", output)
}
