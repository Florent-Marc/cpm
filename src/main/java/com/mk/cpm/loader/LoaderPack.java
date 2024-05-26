package com.mk.cpm.loader;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.zip.ZipEntry;
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
                try {
                    ZipInputStream d = new ZipInputStream(new FileInputStream(f2));
                    ZipEntry entry;
                    while ((entry = d.getNextEntry()) != null) {
                        if (entry.getName().equals("pack_info.dynx")){
                            p.add(f.getName()+" (Soon)");
                        }
                    }
                } catch (IOException e) {
                    throw new RuntimeException(e);
                }
            }
        }
        return p;
    }
}
