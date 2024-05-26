package com.mk.cpm.loader;

import com.mk.cpm.config.Config;

import java.io.*;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

public class LoaderBlock {

    private static List<Block> blocks;


    //get all block in the directory
    public static List<Block> getBlocksByPack(String s){
        if (blocks ==null){
            blocks = new java.util.ArrayList<>();
        }
        List<Block> b = new java.util.ArrayList<>();
        File file = new File(Config.getLastdirectory() +"/"+ s);
        //get all dir in the directory
        //check if file contains .dnxpack
        if (file.getName().contains(".dnxpack")){
            blocks = b;
            return b;
        }
        File[] files = file.listFiles();
        String[] ban = {"assets", "pack_info.dynx"};
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

            }
            fr.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
        return block;
    }



    public static List<Block> getBlocks() {
        return blocks;
    }
}
