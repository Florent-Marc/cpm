package com.mk.cpm.loader;

import com.mk.cpm.config.Config;

import java.io.*;
import java.nio.file.Files;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

public class LoaderBlock {

    private static List<Block> blocks;


    //get all block in the directory
    public static List<Block> getBlocksByPack(Object s){
        if (blocks ==null){
            blocks = new java.util.ArrayList<>();
        }
        List<Block> b = new java.util.ArrayList<>();
        File file = new File(Config.getLastdirectory() +"/"+ s);
        String[] ban = {"assets", "pack_info.dynx"};
        //get all dir in the directory
        //check if file contains .dnxpack
        if (file.getName().contains(".dnxpack")){
            List<Object> objects = new java.util.ArrayList<>();
            objects = (List<Object>) LoaderDynx.loadContentDynxPack(file.getAbsolutePath());
            for (Object o : objects){
                if (o instanceof Block){
                    b.add((Block) o);

                }
            }
            blocks = b;
            return b;
        }
        File[] files = file.listFiles();
        if (files == null){
            blocks = b;
            return b;
        }
        for (File f : files){

            //si ban contient f.getName() alors on continue
            if (java.util.Arrays.asList(ban).contains(f.getName())){
                continue;
            }
            if (f.getName().contains("block_")){
                b.add(createBlock(f));
            }
            if (f.isDirectory()){
                File[] files2 = f.listFiles();
                for (File f2 : files2){
                    if (f2.getName().contains("block_")){
                        b.add(createBlock(f2));
                    }
                    if (f2.isDirectory()){
                        File[] files3 = f2.listFiles();
                        for (File f3 : files3){
                            if (f3.getName().contains("block_")){
                                b.add(createBlock(f3));
                            }
                        }
                    }
                }
            }
        }
        blocks = b;
        return b;
    }

    private static Block createBlock(File f) {
        Block block = new Block();
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
                    block.setName(name);
                }
                if (line.contains("Description")) {
                    String desc = line.split(":")[1];
                    desc = desc.substring(1);
                    block.setDesc(desc);
                }
                if (line.contains("Model")) {
                    String model = line.split(":")[1];
                    model = model.substring(1);
                    block.setModel(model);
                }
                if (line.contains("Scale")) {
                    String scale = line.split(":")[1];
                    scale = scale.substring(1);
                    block.setScale(scale);
                }
                if (line.contains("Translate")) {
                    String translation = line.split(":")[1];
                    translation = translation.substring(1);
                    block.setTranslation(translation);
                }
                if (line.contains("CreativeTab")) {
                    String creativeTab = line.split(":")[1];
                    creativeTab = creativeTab.substring(1);
                    block.setCreativeTab(creativeTab);
                }
                if (line.contains("ItemRotate")) {
                    String itemRotate = line.split(":")[1];
                    itemRotate = itemRotate.substring(1);
                    block.setItemRotation(itemRotate);
                }
                if (line.contains("UseComplexCollisions")) {
                    String useComplexCollision = line.split(":")[1];
                    useComplexCollision = useComplexCollision.substring(1);
                    block.setUseComplexCollision(Boolean.parseBoolean(useComplexCollision));
                }
                if (line.contains("ItemScale")) {
                    String itemScale = line.split(":")[1];
                    itemScale = itemScale.substring(1);
                    block.setItemScale(itemScale);
                }
                if (line.contains("ItemTranslate")) {
                    String itemTranslate = line.split(":")[1];
                    itemTranslate = itemTranslate.substring(1);
                    block.setItemTranslation(itemTranslate);
                }
                /*
                prop_barriere{
                    EmptyMass: 50
                    CenterOfGravityOffset: 0.0 0.0 0.0
                    Bounciness: 0.5
                    DespawnTime: 20000
                    ContinuousCollisionDetection: true
                    SpawnOffset: 0 0.9 0
                }
                 */
                if (line.contains("EmptyMass")) {
                    String emptyMass = line.split(":")[1];
                    emptyMass = emptyMass.substring(1);
                    block.setEmptyMass(emptyMass);
                }
                if (line.contains("CenterOfGravityOffset")) {
                    String centerOfGravityOffset = line.split(":")[1];
                    centerOfGravityOffset = centerOfGravityOffset.substring(1);
                    block.setCenterOfGravityOffset(centerOfGravityOffset);
                }
                if (line.contains("DespawnTime")) {
                    String despawnTime = line.split(":")[1];
                    despawnTime = despawnTime.substring(1);
                    block.setDespawnTime(despawnTime);
                }
                if (line.contains("Item3DRenderLocation")) {
                    String item3DRenderLocation = line.split(":")[1];
                    item3DRenderLocation = item3DRenderLocation.substring(1);
                    block.setItem3DRenderLocation(item3DRenderLocation);
                }
                if (line.contains("IconText")) {
                    String iconText = line.split(":")[1];
                    iconText = iconText.substring(1);
                    block.setIconText(iconText);
                }
                if (line.contains("Textures")) {
                    String textures = line.split(":")[1];
                    textures = textures.substring(1);
                    block.setTextures(textures);
                }
                if (line.contains("Rotate")) {
                    String rotation = line.split(":")[1];
                    rotation = rotation.substring(1);
                    block.setRotation(rotation);
                }
                if (line.contains("RenderDistance")) {
                    String renderDistance = line.split(":")[1];
                    renderDistance = renderDistance.substring(1);
                    block.setRenderDistance(renderDistance);
                }
                if (line.contains("LightLevel")) {
                    String lightLevel = line.split(":")[1];
                    lightLevel = lightLevel.substring(1);
                    block.setLightLevel(lightLevel);
                }
                if (line.contains("Material:")) {
                    String material = line.split(":")[1];
                    material = material.substring(1);
                    block.setMaterial(material);
                }
            }

            fr.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
        return block;
    }
    //create block with buffered reader
    public static Block createBlock(BufferedReader br) {
        Block block = new Block();
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
                    block.setName(name);
                }
                if (line.contains("Description")) {
                    String desc = line.split(":")[1];
                    desc = desc.substring(1);
                    block.setDesc(desc);
                }
                if (line.contains("Model")) {
                    String model = line.split(":")[1];
                    model = model.substring(1);
                    block.setModel(model);
                }
                if (line.contains("Scale")) {
                    String scale = line.split(":")[1];
                    scale = scale.substring(1);
                    block.setScale(scale);
                }
                if (line.contains("Translate")) {
                    String translation = line.split(":")[1];
                    translation = translation.substring(1);
                    block.setTranslation(translation);
                }
                if (line.contains("CreativeTab")) {
                    String creativeTab = line.split(":")[1];
                    creativeTab = creativeTab.substring(1);
                    block.setCreativeTab(creativeTab);
                }
                if (line.contains("ItemRotate")) {
                    String itemRotate = line.split(":")[1];
                    itemRotate = itemRotate.substring(1);
                    block.setItemRotation(itemRotate);
                }
                if (line.contains("UseComplexCollisions")) {
                    String useComplexCollision = line.split(":")[1];
                    useComplexCollision = useComplexCollision.substring(1);
                    block.setUseComplexCollision(Boolean.parseBoolean(useComplexCollision));
                }
                if (line.contains("ItemScale")) {
                    String itemScale = line.split(":")[1];
                    itemScale = itemScale.substring(1);
                    block.setItemScale(itemScale);
                }
                if (line.contains("ItemTranslate")) {
                    String itemTranslate = line.split(":")[1];
                    itemTranslate = itemTranslate.substring(1);
                    block.setItemTranslation(itemTranslate);
                }
                /*
                prop_barriere{
                    EmptyMass: 50
                    CenterOfGravityOffset: 0.0 0.0 0.0
                    Bounciness

                 */
                if (line.contains("EmptyMass")) {
                    String emptyMass = line.split(":")[1];
                    emptyMass = emptyMass.substring(1);
                    block.setEmptyMass(emptyMass);
                }
                if (line.contains("CenterOfGravityOffset")) {
                    String centerOfGravityOffset = line.split(":")[1];
                    centerOfGravityOffset = centerOfGravityOffset.substring(1);
                    block.setCenterOfGravityOffset(centerOfGravityOffset);
                }
                if (line.contains("DespawnTime")) {
                    String despawnTime = line.split(":")[1];
                    despawnTime = despawnTime.substring(1);
                    block.setDespawnTime(despawnTime);
                }
                if (line.contains("Item3DRenderLocation")) {
                    String item3DRenderLocation = line.split(":")[1];
                    item3DRenderLocation = item3DRenderLocation.substring(1);
                    block.setItem3DRenderLocation(item3DRenderLocation);
                }
                if (line.contains("IconText")) {
                    String iconText = line.split(":")[1];
                    iconText = iconText.substring(1);
                    block.setIconText(iconText);
                }
                if (line.contains("Textures")) {
                    String textures = line.split(":")[1];
                    textures = textures.substring(1);
                    block.setTextures(textures);
                }
                if (line.contains("Rotate")) {
                    String rotation = line.split(":")[1];
                    rotation = rotation.substring(1);
                    block.setRotation(rotation);
                }
                if (line.contains("RenderDistance")) {
                    String renderDistance = line.split(":")[1];
                    renderDistance = renderDistance.substring(1);
                    block.setRenderDistance(renderDistance);
                }
                if (line.contains("LightLevel")) {
                    String lightLevel = line.split(":")[1];
                    lightLevel = lightLevel.substring(1);
                    block.setLightLevel(lightLevel);
                }
                if (line.contains("Material:")) {
                    String material = line.split(":")[1];
                    material = material.substring(1);
                    block.setMaterial(material);
                }
            }

        } catch (IOException e) {
            e.printStackTrace();
        }
        return block;
    }



    public static List<Block> getBlocks() {
        return blocks;
    }

    //get file of the block with the name
    public static File getFile(String blockname){
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
                            if (!f3.getName().contains("block_"))continue;
                            if(isBlock(f3, blockname)){
                                return new File(f3.getAbsolutePath());
                            }
                        }
                    }else {
                        if (!f2.getName().contains("block_"))continue;
                        if(isBlock(f2, blockname)){
                            return new File(f2.getAbsolutePath());
                        }
                    }
                }
            }else {
                if (!f.getName().contains("block_"))continue;

                if (isBlock(f, blockname)) {
                    return new File(f.getAbsolutePath());
                }
            }
        }
        return null;
    }


    //methode for get name in file
    public static boolean isBlock(File f, String name) {
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
