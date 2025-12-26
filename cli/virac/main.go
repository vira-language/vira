package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"runtime"

	"github.com/pterm/pterm"
	"github.com/spf13/cobra"
)

var binPath string

func init() {
	osName := runtime.GOOS
	if osName == "linux" {
		binPath = "/usr/lib/vira-lang/bin"
	} else if osName == "windows" {
		programFiles := os.Getenv("ProgramFiles")
		if programFiles == "" {
			programFiles = "C:\\Program Files"
		}
		binPath = filepath.Join(programFiles, "ViraLang", "bin")
	} else {
		pterm.Fatal.Println("Unsupported OS")
		os.Exit(1)
	}
}

func main() {
	var rootCmd = &cobra.Command{
		Use:   "virac [input.vira]",
		Short: "Vira compilation tool",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			compile(args[0])
		},
	}

	if err := rootCmd.Execute(); err != nil {
		pterm.Error.Println(err)
		os.Exit(1)
	}
}

func compile(inputFile string) {
	outputPre := inputFile + ".pre"
	outputObj := inputFile + ".o"

	pterm.DefaultSection.Println("Preprocessing")
	preprocessor := filepath.Join(binPath, "preprocessor")
	if runtime.GOOS == "windows" {
		preprocessor += ".exe"
	}
	cmdPre := exec.Command(preprocessor, inputFile, outputPre)
	if out, err := cmdPre.CombinedOutput(); err != nil {
		handleError(outputPre, string(out))
		os.Exit(1)
	}
	pterm.Success.Println("Preprocessing done")

	pterm.DefaultSection.Println("Parsing and Checking")
	plsa := filepath.Join(binPath, "plsa")
	if runtime.GOOS == "windows" {
		plsa += ".exe"
	}
	cmdPlsa := exec.Command(plsa, outputPre)
	if out, err := cmdPlsa.CombinedOutput(); err != nil {
		handleError(outputPre, string(out))
		os.Exit(1)
	}
	pterm.Success.Println("PLSA done")

	pterm.DefaultSection.Println("Compiling")
	compiler := filepath.Join(binPath, "compiler")
	if runtime.GOOS == "windows" {
		compiler += ".exe"
	}
	cmdComp := exec.Command(compiler, outputPre, outputObj)
	if out, err := cmdComp.CombinedOutput(); err != nil {
		handleError(outputPre, string(out))
		os.Exit(1)
	}
	pterm.Success.Println("Compilation done")

	// Optional: Link to executable
	pterm.DefaultSection.Println("Linking")
	linker := "gcc"
	if runtime.GOOS == "windows" {
		linker = "link.exe" // Adjust as needed
		outputExe := inputFile + ".exe"
		cmdLink := exec.Command(linker, "/OUT:"+outputExe, outputObj) // Simplified
		if out, err := cmdLink.CombinedOutput(); err != nil {
			pterm.Error.Println(string(out))
			os.Exit(1)
		}
	} else {
		outputExe := "a.out" // Or input without ext
		cmdLink := exec.Command(linker, outputObj, "-o", outputExe)
		if out, err := cmdLink.CombinedOutput(); err != nil {
			pterm.Error.Println(string(out))
			os.Exit(1)
		}
	}
	pterm.Success.Println("Linking done")
}

func handleError(sourceFile, errorMsg string) {
	pterm.Error.Println("Error occurred. Running diagnostic...")

	// Parse errorMsg for line, column, message
	// For simplicity, assume errorMsg has "line X, column Y: message"
	// Mock parsing
	line := 1
	column := 1
	message := errorMsg // Full message

	diagnostic := filepath.Join(binPath, "diagnostic")
	if runtime.GOOS == "windows" {
		diagnostic += ".exe"
	}
	cmdDiag := exec.Command(diagnostic,
		"--source", sourceFile,
		"--message", message,
		"--line", string(line + '0'), // Convert to string
		"--column", string(column + '0'),
	)
	if out, err := cmdDiag.CombinedOutput(); err != nil {
		pterm.Error.Println(string(out))
	} else {
		pterm.Info.Println(string(out))
	}
}
