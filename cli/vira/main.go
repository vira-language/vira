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
		Use:   "vira",
		Short: "Vira general CLI tool",
	}

	var compileCmd = &cobra.Command{
		Use:   "compile [input.vira]",
		Short: "Compile a .vira file",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			compile(args[0])
		},
	}

	var updateCmd = &cobra.Command{
		Use:   "update",
		Short: "Update Vira tools",
		Run: func(cmd *cobra.Command, args []string) {
			update()
		},
	}

	rootCmd.AddCommand(compileCmd, updateCmd)
	if err := rootCmd.Execute(); err != nil {
		pterm.Error.Println(err)
		os.Exit(1)
	}
}

func compile(inputFile string) {
	outputPre := inputFile + ".pre"
	outputPlsa := inputFile + ".ast" // Assume some output
	outputDiag := inputFile + ".diag" // Assume

	pterm.DefaultSection.Println("Preprocessing")
	preprocessor := filepath.Join(binPath, "preprocessor")
	if runtime.GOOS == "windows" {
		preprocessor += ".exe"
	}
	cmdPre := exec.Command(preprocessor, inputFile, outputPre)
	if out, err := cmdPre.CombinedOutput(); err != nil {
		pterm.Error.Println(string(out))
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
		pterm.Error.Println(string(out))
		os.Exit(1)
	}
	pterm.Success.Println("PLSA done")

	// Assume diagnostic needs error simulation, but for now skip or mock
	// diagnostic := filepath.Join(binPath, "diagnostic")
	// cmdDiag := exec.Command(diagnostic, "--source", outputPre, "--message", "error", "--line", "1", "--column", "1")
	// if out, err := cmdDiag.CombinedOutput(); err != nil {
	// 	pterm.Error.Println(string(out))
	// 	os.Exit(1)
	// }
	// pterm.Success.Println("Diagnostic done")

	pterm.DefaultSection.Println("Compiling")
	compiler := filepath.Join(binPath, "compiler")
	if runtime.GOOS == "windows" {
		compiler += ".exe"
	}
	outputObj := inputFile + ".o"
	cmdComp := exec.Command(compiler, outputPre, outputObj)
	if out, err := cmdComp.CombinedOutput(); err != nil {
		pterm.Error.Println(string(out))
		os.Exit(1)
	}
	pterm.Success.Println("Compilation done")
}

func update() {
	pterm.DefaultSection.Println("Updating Vira")
	updater := filepath.Join(binPath, "updater")
	if runtime.GOOS == "windows" {
		updater += ".exe"
	}
	cmdUpdate := exec.Command(updater)
	if out, err := cmdUpdate.CombinedOutput(); err != nil {
		pterm.Error.Println(string(out))
		os.Exit(1)
	}
	pterm.Success.Println("Update done")
}
