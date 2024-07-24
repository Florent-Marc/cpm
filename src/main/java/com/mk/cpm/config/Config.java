package com.mk.cpm.config;

import org.json.simple.JSONObject;
import org.json.simple.parser.JSONParser;
import org.json.simple.parser.ParseException;

import java.io.File;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;

public class Config {

    private String lastdirectory = "";
    private boolean debug = false;
    private AddonSettingConfig addonSettingConfig = null;
    private long newRequest = 0;
    private boolean warning = true;

    //json simple create json with user and lastdirectory
    public Config(String user, String lastdirectory) {
        this.lastdirectory = lastdirectory;
    }

    public Config() {

    }

    public void saveConfig() {
        JSONObject obj = new JSONObject();
        obj.put("lastdirectory", getLastdirectory());
        obj.put("debug", isDebug());
        obj.put("newRequest", getnewRequest());
        obj.put("addonSettingsConfig", getAddonSettingConfig().toJson());
        obj.put("warning", warning);
        try {
            File file = new File(System.getenv("APPDATA") + "/cpm/config.json");
            file.getParentFile().mkdirs();
            file.createNewFile();
            Files.write(file.toPath(), obj.toJSONString().getBytes());
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    public AddonSettingConfig getAddonSettingConfig() {
        return addonSettingConfig;
    }


    public void setAddonSettingConfig(AddonSettingConfig addonSettingConfig) {
        this.addonSettingConfig = addonSettingConfig;
    }

    public long getNewRequest() {
        return newRequest;
    }

    public void setNewRequest(long newRequest) {
        this.newRequest = newRequest;
    }

    public void load() {
        //clear le dossier pack

        //load the config from %appdata%/cpm/config.json
        //check if the file exists
        //if not create it
        JSONObject obj = null;
        if (new File(System.getenv("APPDATA") + "/cpm/config.json").exists()) {
            //create the file
            try {
                JSONObject jsonObject = (JSONObject) new JSONParser().parse(new FileReader(System.getenv("APPDATA") + "/cpm/config.json"));
                lastdirectory = (String) jsonObject.get("lastdirectory");
                debug = (boolean) jsonObject.get("debug");
                addonSettingConfig = new AddonSettingConfig().fromJson((JSONObject) jsonObject.get("addonSettingsConfig"));
                if (jsonObject.get("newRequest") != null) {
                    newRequest = (long) jsonObject.get("newRequest");
                }
                if (jsonObject.get("warning") != null) {
                    warning = (boolean) jsonObject.get("warning");
                }
            } catch (IOException | ParseException e) {
                e.printStackTrace();
            }
        } else {
            //read the file
            //json simple
        }
    }


    public String getLastdirectory() {
        return lastdirectory;
    }

    public void setLastdirectory(String lastdirectory) {
        this.lastdirectory = lastdirectory;
    }
    public static String getCachePath() {
        return System.getenv("APPDATA") + "/cpm/cache";
    }

    public boolean isDebug() {
        return debug;
    }

    public void setDebug(boolean debug) {
        this.debug = debug;
    }

    public long getnewRequest() {
        return newRequest;
    }

    public void setnewRequest(long newRequest) {
        this.newRequest = newRequest;
    }

    public boolean isWarning() {
        return warning;
    }

    public void setWarning(boolean warning) {
        this.warning = warning;
    }
}
