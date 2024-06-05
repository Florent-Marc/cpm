package com.mk.cpm.loader;

import com.mk.cpm.config.Config;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;
import java.io.IOException;
import java.util.List;

public class LoaderArmor {

    private static List<Armor> armors;


    //get all block in the directory
    public static List<Armor> getArmorByPack(Object s){
        if (armors ==null){
            armors = new java.util.ArrayList<>();
        }
        List<Armor> b = new java.util.ArrayList<>();
        File file = new File(Config.getLastdirectory() +"/"+ s);
        String[] ban = {"assets", "pack_info.dynx"};
        //get all dir in the directory
        //check if file contains .dnxpack
        if (file.getName().contains(".dnxpack")){
            List<Object> objects = new java.util.ArrayList<>();
            objects = (List<Object>) LoaderDynx.loadContentDynxPack(file.getAbsolutePath());
            for (Object o : objects){
                if (o instanceof Armor){
                    b.add((Armor) o);
                }
            }
            armors = b;
            return b;
        }
        File[] files = file.listFiles();
        if (files == null){
            armors = b;
            return b;
        }
        for (File f : files){

            //si ban contient f.getName() alors on continue
            if (java.util.Arrays.asList(ban).contains(f.getName())){
                continue;
            }
            if (f.getName().contains("armor_")){
                b.add(createArmor(f));
            }
            if (f.isDirectory()){
                File[] files2 = f.listFiles();
                for (File f2 : files2){
                    if (f2.getName().contains("armor_")){
                        b.add(createArmor(f2));
                    }
                    if (f2.isDirectory()){
                        File[] files3 = f2.listFiles();
                        for (File f3 : files3){
                            if (f3.getName().contains("armor_")){
                                b.add(createArmor(f3));
                            }
                        }
                    }
                }
            }
        }
        armors = b;
        return b;
    }

    private static Armor createArmor(File f) {
        Armor a = new Armor();
        try {
            FileReader fr = new FileReader(f);
            BufferedReader br = new BufferedReader(fr);
            StringBuffer sb = new StringBuffer();
            String line;
            while ((line = br.readLine()) != null) {
                //check si elle n'est pas vide
                if (line.isEmpty()){
                    continue;
                }
                sb.append(line);
                sb.append("\n");
                if (line.contains("Name")) {
                    String name = line.split(":")[1];
                    name = name.substring(1);
                    a.setName(name);
                }
                if (line.contains("Description")) {
                    String desc = line.split(":")[1];
                    desc = desc.substring(1);
                    a.setDesc(desc);
                }
                if (line.contains("Model:")) {
                    String model = line.split(":")[1];
                    model = model.substring(1);
                    a.setModel(model);
                }
                if (line.contains("CreativeTab")) {
                    String creativeTab = line.split(":")[1];
                    creativeTab = creativeTab.substring(1);
                    a.setCreativeTab(creativeTab);
                }
                if (line.contains("ItemRotate")) {
                    String itemRotate = line.split(":")[1];
                    itemRotate = itemRotate.substring(1);
                    a.setItemRotation(itemRotate);
                }
                if (line.contains("ItemScale")) {
                    String itemScale = line.split(":")[1];
                    itemScale = itemScale.substring(1);
                    a.setItemScale(itemScale);
                }
                if (line.contains("ItemTranslate")) {
                    String itemTranslate = line.split(":")[1];
                    itemTranslate = itemTranslate.substring(1);
                    a.setItemTranslate(itemTranslate);
                }
                if (line.contains("Item3DRenderLocation")) {
                    String item3DRenderLocation = line.split(":")[1];
                    item3DRenderLocation = item3DRenderLocation.substring(1);
                    a.setItem3DRenderLocation(item3DRenderLocation);
                }
                if (line.contains("IconText")) {
                    String iconText = line.split(":")[1];
                    iconText = iconText.substring(1);
                    a.setIconText(iconText);
                }
                if (line.contains("Textures")) {
                    String textures = line.split(":")[1];
                    textures = textures.substring(1);
                    a.setTextures(textures);
                }
                if (line.contains("ArmorHead")) {
                    String armorHead = line.split(":")[1];
                    armorHead = armorHead.substring(1);
                    a.setArmorHead(armorHead);
                }
                if (line.contains("ArmorBody")) {
                    String armorBody = line.split(":")[1];
                    armorBody = armorBody.substring(1);
                    a.setArmorBody(armorBody);
                }
                if (line.contains("ArmorArms")) {
                    String armorArms = line.split(":")[1];
                    armorArms = armorArms.substring(1);
                    a.setArmorArms(armorArms);
                }
                if (line.contains("ArmorLegs")) {
                    String armorLegs = line.split(":")[1];
                    armorLegs = armorLegs.substring(1);
                    a.setArmorLegs(armorLegs);
                }
                if (line.contains("ArmorFoot")) {
                    String armorFoot = line.split(":")[1];
                    armorFoot = armorFoot.substring(1);
                    a.setArmorFoot(armorFoot);
                }
                if (line.contains("Durability")) {
                    String durability = line.split(":")[1];
                    durability = durability.substring(1);
                    a.setDurability(durability);
                }
                if (line.contains("Textures")) {
                    String textures = line.split(":")[1];
                    textures = textures.substring(1);
                    a.setTextures(textures);
                }
                if (line.contains("Toughness")) {
                    String toughness = line.split(":")[1];
                    toughness = toughness.substring(1);
                    a.setToughness(toughness);
                }
                if (line.contains("DamageReduction")) {
                    String damageReduction = line.split(":")[1];
                    damageReduction = damageReduction.substring(1);
                    a.setDamageReduction(damageReduction);
                }
            }

            fr.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
        return a;
    }
    //create block with buffered reader
    public static Armor createArmor(BufferedReader br) {
        Armor a = new Armor();
        try {
            StringBuffer sb = new StringBuffer();
            String line;
            while ((line = br.readLine()) != null) {
                //check si elle n'est pas vide
                if (line.isEmpty()){
                    continue;
                }
                sb.append(line);
                sb.append("\n");
                if (line.contains("Name")) {
                    String name = line.split(":")[1];
                    name = name.substring(1);
                    a.setName(name);
                }
                if (line.contains("Description")) {
                    String desc = line.split(":")[1];
                    desc = desc.substring(1);
                    a.setDesc(desc);
                }
                if (line.contains("Model:")) {
                    String model = line.split(":")[1];
                    model = model.substring(1);
                    a.setModel(model);
                }
                if (line.contains("CreativeTab")) {
                    String creativeTab = line.split(":")[1];
                    creativeTab = creativeTab.substring(1);
                    a.setCreativeTab(creativeTab);
                }
                if (line.contains("ItemRotate")) {
                    String itemRotate = line.split(":")[1];
                    itemRotate = itemRotate.substring(1);
                    a.setItemRotation(itemRotate);
                }

                if (line.contains("ItemScale")) {
                    String itemScale = line.split(":")[1];
                    itemScale = itemScale.substring(1);
                    a.setItemScale(itemScale);
                }

                if (line.contains("Item3DRenderLocation")) {
                    String item3DRenderLocation = line.split(":")[1];
                    item3DRenderLocation = item3DRenderLocation.substring(1);
                    a.setItem3DRenderLocation(item3DRenderLocation);
                }
                if (line.contains("IconText")) {
                    String iconText = line.split(":")[1];
                    iconText = iconText.substring(1);
                    a.setIconText(iconText);
                }
                if (line.contains("Textures")) {
                    String textures = line.split(":")[1];
                    textures = textures.substring(1);
                    a.setTextures(textures);
                }
                if (line.contains("ArmorHead")) {
                    String armorHead = line.split(":")[1];
                    armorHead = armorHead.substring(1);
                    a.setArmorHead(armorHead);
                }
                if (line.contains("ArmorBody")) {
                    String armorBody = line.split(":")[1];
                    armorBody = armorBody.substring(1);
                    a.setArmorBody(armorBody);
                }
                if (line.contains("ArmorArms")) {
                    String armorArms = line.split(":")[1];
                    armorArms = armorArms.substring(1);
                    a.setArmorArms(armorArms);
                }
                if (line.contains("ArmorLegs")) {
                    String armorLegs = line.split(":")[1];
                    armorLegs = armorLegs.substring(1);
                    a.setArmorLegs(armorLegs);
                }
                if (line.contains("ArmorFoot")) {
                    String armorFoot = line.split(":")[1];
                    armorFoot = armorFoot.substring(1);
                    a.setArmorFoot(armorFoot);
                }
                if (line.contains("Durability")) {
                    String durability = line.split(":")[1];
                    durability = durability.substring(1);
                    a.setDurability(durability);
                }
                if (line.contains("Textures")) {
                    String textures = line.split(":")[1];
                    textures = textures.substring(1);
                    a.setTextures(textures);
                }
                if (line.contains("Toughness")) {
                    String toughness = line.split(":")[1];
                    toughness = toughness.substring(1);
                    a.setToughness(toughness);
                }
                if (line.contains("DamageReduction")) {
                    String damageReduction = line.split(":")[1];
                    damageReduction = damageReduction.substring(1);
                    a.setDamageReduction(damageReduction);
                }
            }

        } catch (IOException e) {
            e.printStackTrace();
        }
        return a;
    }



    public static List<Armor> getArmor() {
        return armors;
    }

    //get file of the block with the name
    public static File getFile(String armorname){
        File dir = new File(Config.getLastdirectory());
        File[] files = dir.listFiles();
        if (files == null){
            return null;
        }
        for (File f : files){
            if (f.getName().contains("assets") || f.getName().contains("pack_info.dynx")){
                continue;
            }
            if (f.isDirectory()){
                File[] files2 = f.listFiles();
                if (files2 == null){
                    return null;
                }
                for (File f2 : files2){
                    if (f2.isDirectory()){
                        File[] files3 = f2.listFiles();
                        if (files3 == null){
                            return null;
                        }
                        for (File f3 : files3){
                            if (!f3.getName().contains("armor_"))continue;
                            if(isArmor(f3, armorname)){
                                return new File(f3.getAbsolutePath());
                            }
                        }
                    }else {
                        if (!f2.getName().contains("armor_"))continue;
                        if(isArmor(f2, armorname)){
                            return new File(f2.getAbsolutePath());
                        }
                    }
                }
            }else {
                if (!f.getName().contains("armor_"))continue;

                if (isArmor(f, armorname)) {
                    return new File(f.getAbsolutePath());
                }
            }
        }
        return null;
    }


    //methode for get name in file
    public static boolean isArmor(File f, String name) {
        try {
            FileReader fr = new FileReader(f);
            BufferedReader br = new BufferedReader(fr);
            String line;
            while ((line = br.readLine()) != null) {
                if (line.contains("Name")) {
                    String name2 = line.split(":")[1];
                    name2 = name2.substring(1);
                    if (name2.equals(name)) {
                        return true;
                    }
                }
            }
            fr.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
        return false;
    }
}
