package com.mk.cpm.loader;

import org.json.simple.JSONObject;
import org.json.simple.parser.JSONParser;
import org.json.simple.parser.ParseException;

import java.io.File;
import java.io.FileReader;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Formattable;
import java.util.List;

public class LoaderAddon {

    private List<Object> vehicles;
    private List<Object> blocks;
    private List<Object> items;
    private List<Object> armors;


    public void load() {
        FileReader fileReader = null;
        vehicles = new ArrayList<>();
        blocks = new ArrayList<>();
        items = new ArrayList<>();
        armors = new ArrayList<>();
        File file = new File(System.getenv("APPDATA") + "/cpm/addons/");
        List<String> list = new ArrayList<>();
        for (File f : file.listFiles()) {
            list.add(f.getName());
        }
        for (String s : list) {
            try {
                fileReader = new FileReader(System.getenv("APPDATA") + "/cpm/addons/" + s);
                Object obj = new JSONParser().parse(fileReader);
                JSONObject jsonObject = (JSONObject) obj;
                if ((JSONObject) jsonObject.get("vehicules") != null) {
                    JSONObject vehicles = (JSONObject) jsonObject.get("vehicules");
                    for (Object vehicle : vehicles.keySet()) {
                        MultiParam multiParam = new MultiParam((List<String>) vehicles.get(vehicle), (String) vehicle);
                        this.vehicles.add(multiParam);
                    }
                }
                if ((JSONObject) jsonObject.get("blocks") != null) {
                    JSONObject blocks = (JSONObject) jsonObject.get("blocks");
                    for (Object block : blocks.keySet()) {
                        MultiParam multiParam = new MultiParam((List<String>) blocks.get(block), (String) block);
                        this.blocks.add(multiParam);
                    }
                }
                if ((JSONObject) jsonObject.get("items") != null) {
                    JSONObject items = (JSONObject) jsonObject.get("items");
                    for (Object item : items.keySet()) {
                        MultiParam multiParam = new MultiParam((List<String>) items.get(item), (String) item);
                        this.items.add(multiParam);
                    }
                }
                if ((JSONObject) jsonObject.get("armors") != null) {
                    JSONObject armors = (JSONObject) jsonObject.get("armors");
                    for (Object armor : armors.keySet()) {
                        MultiParam multiParam = new MultiParam((List<String>) armors.get(armor), (String) armor);
                        this.armors.add(multiParam);
                    }
                }
            } catch (IOException | ParseException e) {
                throw new RuntimeException(e);
            }
        }
    }

    public static void main(String[] args) {
        LoaderAddon loaderAddon = new LoaderAddon();
        loaderAddon.load();
    }

    public List<Object> getVehicles() {
        return vehicles;
    }
}
