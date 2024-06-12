package com.mk.cpm.loader;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Enumeration;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipException;
import java.util.zip.ZipFile;
import java.util.zip.ZipInputStream;

public class LoaderPack {

    //get all dossier in String if contains file pack_info.dynx
    public List<String> getPacks(File directory){
        List<String> p = new ArrayList<>();
        if (!directory.exists()){
            return p;
        }
        for (File f : directory.listFiles()){
            if (f.isDirectory()){
                for (File f2 : f.listFiles()){
                    if (f2.getName().equals("pack_info.dynx")){
                        p.add(f.getName());
                    }
                }
            }
            if (f.getName().contains(".dnxpack")){
                File f2 = new File(f.getAbsolutePath());
                try (ZipFile zipFile = new ZipFile(f2)) {
                    Enumeration<? extends ZipEntry> entries = zipFile.entries();

                    while (entries.hasMoreElements()) {
                        ZipEntry entry = entries.nextElement();
                        if(entry.getName().equals("pack_info.dynx")||entry.getName().equals("MpsRepositories.dnx")){
                            if (!p.contains(f.getName())) {
                                p.add(f.getName());
                            }
                        }
                    }
                } catch (IOException e) {
                    throw new RuntimeException(e);
                }
            }
        }
        return p;
    }

    public static void deleteDirectory(File directory) {
        File[] allContents = directory.listFiles();
        if (allContents != null) {
            for (File file : allContents) {
                deleteDirectory(file);
            }
        }
        directory.delete();
    }
}
