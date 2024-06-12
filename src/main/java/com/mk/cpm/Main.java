package com.mk.cpm;

import com.mk.cpm.config.Config;

import java.io.File;

public class Main {

    public static boolean isDynxPack = false;


    public static void main(String[] args) {
        File folder = new File(Config.getCachePath()+"/pack/");
        deleteDirectory(folder);
        Config.load();
        HelloApplication.main(args);
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
