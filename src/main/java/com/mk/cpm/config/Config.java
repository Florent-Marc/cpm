package com.mk.cpm.config;

import org.json.simple.JSONObject;
import org.json.simple.parser.JSONParser;
import org.json.simple.parser.ParseException;

import java.io.File;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;

public class Config {

    private static String user = "";
    private static String lastdirectory = "";
    private static boolean debug = false;

    //json simple create json with user and lastdirectory
    public Config(String user, String lastdirectory) {
        this.user = user;
        this.lastdirectory = lastdirectory;
    }

    public static void saveConfig() {
        //save the config to %appdata%/cpm/config.json
        JSONObject obj = new JSONObject();
        obj.put("user", getUser());
        obj.put("lastdirectory", getLastdirectory());
        obj.put("debug", isDebug());
        try {
            File file = new File(System.getenv("APPDATA") + "/cpm/config.json");
            file.getParentFile().mkdirs();
            file.createNewFile();
            FileWriter fileWriter = new FileWriter(file);
            fileWriter.write(obj.toJSONString());
            fileWriter.flush();
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    public static void load() {
        //clear le dossier pack

        //load the config from %appdata%/cpm/config.json
        //check if the file exists
        //if not create it
        JSONObject obj = null;
        if (new File(System.getenv("APPDATA") + "/cpm/config.json").exists()) {
            //create the file
            try {
                JSONObject jsonObject = (JSONObject) new JSONParser().parse(new FileReader(System.getenv("APPDATA") + "/cpm/config.json"));
                user = (String) jsonObject.get("user");
                lastdirectory = (String) jsonObject.get("lastdirectory");
                debug = (boolean) jsonObject.get("debug");
            } catch (IOException | ParseException e) {
                e.printStackTrace();
            }
        } else {
            //read the file
            //json simple
        }
    }

    public static String getUser() {
        return user;
    }

    public static String getLastdirectory() {
        return lastdirectory;
    }

    public static void setUser(String user) {
        Config.user = user;
    }

    public static void setLastdirectory(String lastdirectory) {
        Config.lastdirectory = lastdirectory;
    }
    //get path cache
    public static String getCachePath() {
        return System.getenv("APPDATA") + "/cpm/cache";
    }

    public static boolean isDebug() {
        return debug;
    }

    public static void setDebug(boolean debug) {
        Config.debug = debug;
    }
}
