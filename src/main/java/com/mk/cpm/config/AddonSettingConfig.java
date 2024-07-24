package com.mk.cpm.config;

import org.json.simple.JSONObject;

import java.util.ArrayList;
import java.util.List;

public class AddonSettingConfig {
    private List<String> enabledAddons = new ArrayList<>();
    private String lastFolder = "";

    public AddonSettingConfig(List<String> enabledAddons, String lastFolder) {
        this.enabledAddons = enabledAddons;
        this.lastFolder = lastFolder;
    }

    public AddonSettingConfig() {

    }

    public List<String> getEnabledAddons() {
        return enabledAddons;
    }

    public void setEnabledAddons(List<String> enabledAddons) {
        this.enabledAddons = enabledAddons;
    }

    public void addEnabledAddon(String enabledAddon) {
        enabledAddons.add(enabledAddon);
    }

    public void removeEnabledAddon(String enabledAddon) {
        enabledAddons.remove(enabledAddon);
    }

    public String getLastFolder() {
        return lastFolder;
    }

    public void setLastFolder(String lastFolder) {
        this.lastFolder = lastFolder;
    }

    public AddonSettingConfig fromJson(JSONObject jsonObject) {
        if(jsonObject == null) return new AddonSettingConfig();
        List<String> enabledAddons = (List<String>) jsonObject.get("enabledAddons");
        if(enabledAddons == null) enabledAddons = new ArrayList<>();
        String lastFolder = (String) jsonObject.get("lastFolder");
        if(lastFolder == null) lastFolder = "";
        return new AddonSettingConfig(enabledAddons, lastFolder);
    }

    public JSONObject toJson() {
        JSONObject jsonObject = new JSONObject();
        jsonObject.put("enabledAddons", enabledAddons);
        jsonObject.put("lastFolder", lastFolder);
        return jsonObject;
    }
}
