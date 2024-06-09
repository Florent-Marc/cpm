package com.mk.cpm.loader;

import com.mk.cpm.config.Config;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.object.Item;
import com.mk.cpm.loader.object.Vehicul;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

public class Loader {


    private static List<Object> objectsList;


    public static void main(String[] args) {

        //file path pack/AmenagementUrbain/vehicles/
        String path = "pack/AmenagementUrbain/vehicles/mercedesclasscla2013/";
        //get all files in the directory
        File folder = new File(path);
        File[] listOfFiles = folder.listFiles();
        if (listOfFiles == null) {
            return;
        }
        for (File file : listOfFiles) {
            if (file.isFile()) {
                //load the file
                if (file.getName().contains("vehicle_")) {
                    Vehicul vehicul = new Vehicul();
                    vehicul.load(file);
                    System.out.println(Arrays.toString(vehicul.getInfos().toArray()));
                }
                if (file.getName().contains("block_")) {
                    Block block = new Block();
                    block.load(file);
                }
            }
        }
    }

    //get all and return object item ,block ,armor ,vehicul with Config.getLastdirectory()
    public static List<Object> getBlocksByPack(String s){
        if (s == null){
            return new ArrayList<>();
        }
        List<Object> objects = new ArrayList<>();
        File file = new File(Config.getLastdirectory() +"/"+ s);
        String[] ban = {"assets", "pack_info.dynx"};
        //get all dir in the directory
        //check if file contains .dnxpack
        if (file.getName().contains(".dnxpack")){
            objects = (List<Object>) LoaderDynx.loadContentDynxPack(file.getAbsolutePath());
            for (Object o : objects){
                if (o instanceof Block){
                    Block block = (Block) o;
                    System.out.println(block.getName());

                }
            }
            return objects;
        }
        File[] files = listFiles(file.getAbsolutePath()).stream().map(Path::toFile).toArray(File[]::new);
        if (files == null){
            return objects;
        }
        //get all files in the directory et tous les fichier dans les sous dossier
        for (File f : files){
            //si ban contient f.getName() alors on continue
            if (java.util.Arrays.asList(ban).contains(f.getName())){
                continue;
            }
            if (f.getName().contains("block_")){
                Block block = new Block();
                block.load(f);
                objects.add(block);
            }
            if (f.getName().contains("armor_")){
                Armor block = new Armor();
                block.load(f);
                objects.add(block);
            }
            if (f.getName().contains("vehicle_")){
                Vehicul vehicul = new Vehicul();
                vehicul.load(f);
                objects.add(vehicul);
            }
            if (f.getName().contains("item_")){
                Item item = new Item();
                item.load(f);
                objects.add(item);
            }
        }
        objectsList = objects;
        return objects;
    }

    public static List<Path> listFiles(String directory) {
        try (Stream<Path> paths = Files.walk(Paths.get(directory))) {
            return paths.filter(Files::isRegularFile)
                    .collect(Collectors.toList());
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    public static List<Object> getObject() {
        return objectsList;
    }
}
