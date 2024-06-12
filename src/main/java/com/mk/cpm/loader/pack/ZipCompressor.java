package com.mk.cpm.loader.pack;

import com.mk.cpm.config.Config;

import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

public class ZipCompressor {

    public static void compressFolder(String sourceDirPath, String zipFilePath) throws IOException {
        long start = System.currentTimeMillis();
        Path sourceDir = Paths.get(sourceDirPath);
        try (FileOutputStream fos = new FileOutputStream(zipFilePath);
             ZipOutputStream zos = new ZipOutputStream(fos)) {

            Files.walk(sourceDir)
                    .filter(path -> !Files.isDirectory(path))
                    .forEach(path -> {
                        ZipEntry zipEntry = new ZipEntry(sourceDir.relativize(path).toString());
                        try {
                            zos.putNextEntry(zipEntry);
                            Files.copy(path, zos);
                            zos.closeEntry();
                        } catch (IOException e) {
                            System.err.println("Failed to add file to ZIP: " + path);
                            e.printStackTrace();
                        }
                    });
        }
        long end = System.currentTimeMillis();
        System.out.println("Compression time: " + (end - start) + " ms");
    }

    public static void main(String[] args) {
        String sourceDirPath = Config.getCachePath() + "/pack";
        String zipFilePath = Config.getCachePath() + "/build/test.dnxpack";

        try {
            compressFolder(sourceDirPath, zipFilePath);
            System.out.println("Compression completed successfully!");
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
