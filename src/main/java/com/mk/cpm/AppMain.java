package com.mk.cpm;

import com.mk.cpm.config.Config;

import java.io.File;

public class AppMain {

    public static boolean isDynxPack = false;
    public static String CurrentAdd = "";
    public static Object o;

    public static Config config = new Config();

    public static String DYNAMX_PATH = "C:\\Users\\gabid\\Desktop\\cpm\\lib\\DynamX-4.1.0-dev19-all-deobf.jar";


    public static void main(String[] args) {
        File folder = new File(Config.getCachePath()+"/pack/");
        deleteDirectory(folder);
        config.load();
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
