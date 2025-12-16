package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/pterm/pterm"
	"github.com/schollz/progressbar/v3"
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
	pterm.Info.Println("Starting Vira Updater...")

	versionFile := libDir + "/version.json"
	localData, err := os.ReadFile(versionFile)
	if err != nil {
		pterm.Error.Println("No local version found. Please install Vira first.")
		return
	}

	var localVer []string
	err = json.Unmarshal(localData, &localVer)
	if err != nil {
		pterm.Error.Println("Invalid local version file.")
		return
	}
	localV := localVer[0]

	remoteURL := "https://raw.githubusercontent.com/vira-language/vira/main/repository/vira-version.json"
	resp, err := http.Get(remoteURL)
	if err != nil {
		pterm.Error.Printf("Failed to fetch remote version: %v\n", err)
		return
	}
	defer resp.Body.Close()

	remoteData, err := io.ReadAll(resp.Body)
	if err != nil {
		pterm.Error.Println("Failed to read remote version.")
		return
	}

	var remoteVer []string
	err = json.Unmarshal(remoteData, &remoteVer)
	if err != nil {
		pterm.Error.Println("Invalid remote version.")
		return
	}
	remoteV := remoteVer[0]

	if compareVersions(remoteV, localV) <= 0 {
		pterm.Success.Printf("Already up to date (version %s)\n", localV)
		return
	}

	pterm.Warning.Printf("Update available: %s (current: %s)\n", remoteV, localV)
	pterm.Info.Println("Performing update...")

	// Remove existing files
	viraBin := systemBinDir + "/vira" + exe
	viracBin := systemBinDir + "/virac" + exe
	os.Remove(viraBin)
	os.Remove(viracBin)

	// Remove all in binDir
	entries, err := os.ReadDir(binDir)
	if err == nil {
		for _, entry := range entries {
			os.Remove(filepath.Join(binDir, entry.Name()))
		}
	}

	// Create dirs if needed
	os.MkdirAll(systemBinDir, 0755)
	os.MkdirAll(binDir, 0755)

	versionTag := "v" + remoteV
	baseURL := "https://github.com/vira-language/vira/releases/download/" + versionTag + "/"

	files := []string{"vira", "virac", "compiler", "vm", "translator", "interpreter", "updater", "plsa"}

	for _, f := range files {
		fileURL := baseURL + f
		if runtime.GOOS == "windows" {
			fileURL += ".exe"
		}

		pterm.Info.Printf("Downloading %s...\n", f)

		resp, err := http.Get(fileURL)
		if err != nil {
			pterm.Error.Printf("Failed to download %s: %v\n", f, err)
			return
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			pterm.Error.Printf("Failed to download %s: status %d\n", f, resp.StatusCode)
			return
		}

		var path string
		if f == "vira" || f == "virac" {
			path = systemBinDir + "/" + f + exe
		} else {
			path = binDir + "/" + f + exe
		}

		out, err := os.Create(path)
		if err != nil {
			pterm.Error.Printf("Failed to create file %s: %v\n", path, err)
			return
		}

		bar := progressbar.DefaultBytes(
			resp.ContentLength,
			"Downloading",
		)

		_, err = io.Copy(io.MultiWriter(out, bar), resp.Body)
		if err != nil {
			pterm.Error.Printf("Failed to write %s: %v\n", f, err)
			out.Close()
			return
		}
		out.Close()

		if runtime.GOOS != "windows" {
			err = os.Chmod(path, 0755)
			if err != nil {
				pterm.Warning.Printf("Failed to chmod %s: %v\n", path, err)
			}
		}
	}

	// Write new version
	err = os.WriteFile(versionFile, remoteData, 0644)
	if err != nil {
		pterm.Error.Printf("Failed to write new version: %v\n", err)
		return
	}

	pterm.Success.Printf("Updated to version %s\n", remoteV)
}

// Simple version compare (assuming x.y.z)
func compareVersions(v1, v2 string) int {
	parts1 := strings.Split(v1, ".")
	parts2 := strings.Split(v2, ".")

	maxLen := len(parts1)
	if len(parts2) > maxLen {
		maxLen = len(parts2)
	}

	for i := 0; i < maxLen; i++ {
		p1 := 0
		if i < len(parts1) {
			fmt.Sscan(parts1[i], &p1)
		}
		p2 := 0
		if i < len(parts2) {
			fmt.Sscan(parts2[i], &p2)
		}
		if p1 > p2 {
			return 1
		} else if p1 < p2 {
			return -1
		}
	}
	return 0
}
