package com.mk.cpm.loader;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.Enumeration;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

public class LoaderDynx {

    public static Object loadContentDynxPack(String path) {
        List<Object> objects = new ArrayList<>();
        try (ZipFile zipFile = new ZipFile(path)) {
            Enumeration<? extends ZipEntry> entries = zipFile.entries();

            while (entries.hasMoreElements()) {
                ZipEntry entry = entries.nextElement();
                if (entry.getName().contains("block_")) {
                    try (InputStream inputStream = zipFile.getInputStream(entry);
                         BufferedReader reader = new BufferedReader(new InputStreamReader(inputStream))) {
                        objects.add(LoaderBlock.createBlock(reader));
                        String line;
                        while ((line = reader.readLine()) != null) {
                            System.out.println(line);
                        }
                    }
                }
                if (entry.getName().contains("armor_")) {
                    try (InputStream inputStream = zipFile.getInputStream(entry);
                         BufferedReader reader = new BufferedReader(new InputStreamReader(inputStream))) {
                        objects.add(LoaderArmor.createArmor(reader));
                        String line;
                        while ((line = reader.readLine()) != null) {
                            System.out.println(line);
                        }
                    }
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }

        return objects;
    }

    public static void main(String[] args) {
        LoaderDynx loaderDynx = new LoaderDynx();
        loaderDynx.loadContentDynxPack("C:\\Users\\flore\\OneDrive\\Bureau\\Dev\\cpm\\pack\\entreprisepack.dnxpack");
    }

}
