package main

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
)

func main() {
	if err := runUpdater(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
	fmt.Println("Update check complete.")
}

func runUpdater() error {
	osName := runtime.GOOS
	var viraDir, binDir, sysBinDir, zipName string

	if osName == "linux" {
		viraDir = "/usr/lib/vira-lang"
		binDir = filepath.Join(viraDir, "bin")
		sysBinDir = "/usr/bin"
		zipName = "bin-linux.zip"
	} else if osName == "windows" {
		programFiles := os.Getenv("ProgramFiles")
		if programFiles == "" {
			programFiles = "C:\\Program Files"
		}
		viraDir = filepath.Join(programFiles, "ViraLang")
		binDir = filepath.Join(viraDir, "bin")
		sysBinDir = filepath.Join(os.Getenv("SystemRoot"), "System32") // Note: Requires admin privileges
		zipName = "bin-windows.zip"
	} else {
		return fmt.Errorf("unsupported OS: %s", osName)
	}

	versionFile := filepath.Join(viraDir, "version.json")

	// Read local version
	localVersion, err := readVersion(versionFile)
	if err != nil {
		return fmt.Errorf("failed to read local version: %v", err)
	}

	// Download remote version
	remoteURL := "https://raw.githubusercontent.com/vira-language/vira/main/repository/vira-version.json"
	remoteVersionData, err := downloadFileToBytes(remoteURL)
	if err != nil {
		return fmt.Errorf("failed to download remote version: %v", err)
	}

	var remoteVersions []string
	if err := json.Unmarshal(remoteVersionData, &remoteVersions); err != nil || len(remoteVersions) == 0 {
		return fmt.Errorf("invalid remote version JSON: %v", err)
	}
	remoteVersion := remoteVersions[0]

	// Compare versions
	if !isNewerVersion(remoteVersion, localVersion) {
		fmt.Printf("Current version %s is up to date.\n", localVersion)
		return nil
	}

	fmt.Printf("New version %s available (current: %s). Updating...\n", remoteVersion, localVersion)

	// Download zip
	zipURL := fmt.Sprintf("https://github.com/vira-language/vira/releases/download/v%s/%s", remoteVersion, zipName)
	zipData, err := downloadFileToBytes(zipURL)
	if err != nil {
		return fmt.Errorf("failed to download zip: %v", err)
	}

	// Unzip
	if err := unzipBytes(zipData, binDir, sysBinDir, osName); err != nil {
		return fmt.Errorf("failed to unzip: %v", err)
	}

	// Update local version
	if err := writeVersion(versionFile, remoteVersion); err != nil {
		return fmt.Errorf("failed to update local version: %v", err)
	}

	fmt.Println("Update successful.")
	return nil
}

func readVersion(filePath string) (string, error) {
	data, err := os.ReadFile(filePath)
	if err != nil {
		return "", err
	}
	var versions []string
	if err := json.Unmarshal(data, &versions); err != nil || len(versions) == 0 {
		return "", fmt.Errorf("invalid version JSON")
	}
	return versions[0], nil
}

func writeVersion(filePath string, version string) error {
	data, err := json.Marshal([]string{version})
	if err != nil {
		return err
	}
	return os.WriteFile(filePath, data, 0644)
}

func downloadFileToBytes(url string) ([]byte, error) {
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("bad status: %s", resp.Status)
	}
	return io.ReadAll(resp.Body)
}

func unzipBytes(data []byte, binDir, sysBinDir, osName string) error {
	r, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		return err
	}

	if err := os.MkdirAll(binDir, 0755); err != nil {
		return err
	}
	if err := os.MkdirAll(sysBinDir, 0755); err != nil {
		return err
	}

	for _, f := range r.File {
		if f.FileInfo().IsDir() {
			continue
		}

		fileName := f.Name
		baseName := filepath.Base(fileName)
		targetDir := binDir

		exeSuffix := ""
		if osName == "windows" {
			exeSuffix = ".exe"
		}

		if strings.EqualFold(baseName, "vira"+exeSuffix) || strings.EqualFold(baseName, "virac"+exeSuffix) {
			targetDir = sysBinDir
		}

		targetPath := filepath.Join(targetDir, baseName)

		outFile, err := os.OpenFile(targetPath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, f.Mode())
		if err != nil {
			return err
		}
		defer outFile.Close()

		rc, err := f.Open()
		if err != nil {
			return err
		}
		defer rc.Close()

		_, err = io.Copy(outFile, rc)
		if err != nil {
			return err
		}
	}

	return nil
}

func isNewerVersion(remote, local string) bool {
	remoteParts := strings.Split(remote, ".")
	localParts := strings.Split(local, ".")

	maxLen := len(remoteParts)
	if len(localParts) > maxLen {
		maxLen = len(localParts)
	}

	for i := 0; i < maxLen; i++ {
		var remoteNum, localNum int
		if i < len(remoteParts) {
			remoteNum, _ = strconv.Atoi(remoteParts[i])
		}
		if i < len(localParts) {
			localNum, _ = strconv.Atoi(localParts[i])
		}
		if remoteNum > localNum {
			return true
		} else if remoteNum < localNum {
			return false
		}
	}
	return false
}
