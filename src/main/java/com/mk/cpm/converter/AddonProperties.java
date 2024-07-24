package com.mk.cpm.converter;

import org.json.simple.JSONObject;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class AddonProperties {

    public static HashMap<String, String> CPM_DEFINITIONS = new HashMap<String, String>() {{
        put("WHEELED_VEHICLES", "vehicul");
        put("TRAILERS", "trailer");
        put("BOATS", "boat");
        put("HELICOPTER", "helicopter");
        put("ITEMS", "item");
        put("ARMORS", "armor");
        put("BLOCKS", "block");
        put("PROPS", "prop");
        put("WHEELS", "wheel");
        put("CAR_ENGINES", "carEngine");
        put("HELICOPTER_ENGINES", "helicopterEngine");
    }};

    public String name;
    public String version;
    public AddonConverterState state;
    public HashMap<String, List<String>> vehiclesProperties;
    public HashMap<String, List<String>> trailerProperties;
    public HashMap<String, List<String>> boatProperties;
    public HashMap<String, List<String>> helicopterProperties;
    public HashMap<String, List<String>> itemProperties;
    public HashMap<String, List<String>> armorProperties;
    public HashMap<String, List<String>> blockProperties;
    public HashMap<String, List<String>> propProperties;
    public HashMap<String, List<String>> wheelProperties;
    public HashMap<String, List<String>> carEnginesProperties;
    public HashMap<String, List<String>> helicopterEngineProperties;

    public AddonProperties() {
        vehiclesProperties = new HashMap<>();
        trailerProperties = new HashMap<>();
        boatProperties = new HashMap<>();
        helicopterProperties = new HashMap<>();
        itemProperties = new HashMap<>();
        armorProperties = new HashMap<>();
        blockProperties = new HashMap<>();
        propProperties = new HashMap<>();
        wheelProperties = new HashMap<>();
        carEnginesProperties = new HashMap<>();
        helicopterEngineProperties = new HashMap<>();
    }

    @Override
    public String toString() {
        return "AddonProperties{" + '\n' +
                "- name='" + name + '\'' + '\n' +
                "- version='" + version + '\'' +'\n' +
                "- state=" + state + '\n' +
                "- vehiclesProperties=" + showHashMap(vehiclesProperties) +
                "- trailerProperties=" + showHashMap(trailerProperties) +
                "- boatProperties=" + showHashMap(boatProperties) +
                "- helicopterProperties=" + showHashMap(helicopterProperties) +
                "- itemProperties=" + showHashMap(itemProperties) +
                "- armorProperties=" + showHashMap(armorProperties) +
                "- blockProperties=" + showHashMap(blockProperties) +
                "- propProperties=" + showHashMap(propProperties) +
                "- wheelProperties=" + showHashMap(wheelProperties) +
                "- carEnginesProperties=" + showHashMap(carEnginesProperties) +
                "- helicopterEngineProperties=" + showHashMap(helicopterEngineProperties) +
                '}';
    }

    private String showHashMap(HashMap<String, List<String>> map) {
        StringBuilder sb = new StringBuilder();
        sb.append("\n");
        for (Map.Entry<String, List<String>> entry : map.entrySet()) {
            System.out.println(entry.getKey());
            for (String s : entry.getValue()) {
                if (entry.getValue().indexOf(s) == entry.getValue().size() - 1) {
                    sb.append("   └─").append(s).append("\n");
                } else {
                    sb.append("   ├─").append(s).append("\n");
                }
            }
            System.out.println(" ");
        }
        return sb.append("\n").toString();
    }

    public String toJson() {
        JSONObject jsonObject = new JSONObject();
        jsonObject.put("name", name);
        jsonObject.put("version", version);
        jsonObject.put("vehicul", vehiclesProperties);
        jsonObject.put("trailer", trailerProperties);
        jsonObject.put("boat", boatProperties);
        jsonObject.put("helicopter", helicopterProperties);
        jsonObject.put("item", itemProperties);
        jsonObject.put("armor", armorProperties);
        jsonObject.put("block", blockProperties);
        jsonObject.put("prop", propProperties);
        jsonObject.put("wheel", wheelProperties);
        jsonObject.put("carEngine", carEnginesProperties);
        jsonObject.put("helicopterEngine", helicopterEngineProperties);
        return jsonObject.toJSONString();
    }
}
