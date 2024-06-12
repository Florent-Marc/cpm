package com.mk.cpm.loader.pack;

import com.mk.cpm.config.Config;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

public class ZipExtractor {

    public static void extractZip(String zipFilePath, String destDir) throws IOException {
        long start = System.currentTimeMillis();
        Path destDirPath = Paths.get(destDir);

        // Create directories if they do not exist
        if (!Files.exists(destDirPath)) {
            Files.createDirectories(destDirPath);
        }

        try (InputStream fis = Files.newInputStream(Paths.get(zipFilePath));
             ZipInputStream zis = new ZipInputStream(fis)) {

            ZipEntry zipEntry = zis.getNextEntry();
            while (zipEntry != null) {
                Path newFilePath = destDirPath.resolve(zipEntry.getName());

                if (zipEntry.isDirectory()) {
                    // If the entry is a directory, create the directory
                    Files.createDirectories(newFilePath);
                } else {
                    // If the entry is a file, create the parent directories if they do not exist
                    if (!Files.exists(newFilePath.getParent())) {
                        Files.createDirectories(newFilePath.getParent());
                    }

                    // Extract the file
                    try (FileOutputStream fos = new FileOutputStream(newFilePath.toFile())) {
                        byte[] buffer = new byte[1024];
                        int len;
                        while ((len = zis.read(buffer)) > 0) {
                            fos.write(buffer, 0, len);
                        }
                    }
                }

                // Move to the next entry in the ZIP file
                zipEntry = zis.getNextEntry();
            }

            // Close the last ZIP entry
            zis.closeEntry();
        }
        long end = System.currentTimeMillis();
        System.out.println("Extraction time: " + (end - start) + " ms");
    }

    public static void main(String[] args) {
        String zipFilePath = "pack/entreprisepack.dnxpack";
        String destDir = Config.getCachePath()+"/pack";

        try {
            extractZip(zipFilePath, destDir);
            System.out.println("Extraction completed successfully!");
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
