package com.mk.cpm.loader;

import com.mk.cpm.Main;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.object.Item;
import com.mk.cpm.loader.object.Vehicul;
import com.mk.cpm.loader.pack.ZipExtractor;

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
        File file;
        if (Main.isDynxPack){
            try {
                ZipExtractor.extractZip(Config.getLastdirectory()+ "/" + s, Config.getCachePath()+"/pack/"+s);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
            file = new File(Config.getCachePath()+"/pack/"+s);
        }else {
            file = new File(Config.getLastdirectory() + "/" + s);
        }
        String[] ban = {"assets", "pack_info.dynx"};
        //get all dir in the directory
        //check if file contains .dnxpack
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

    //get path of obj with name of obj and packname
    public static String getPath(String name, String packname){
        String path;
        if (packname.contains(".dnxpack")){
            path = Config.getCachePath() + "/pack/" + packname;
        }else {
            path = Config.getLastdirectory() + "/" + packname;
        }
        File f = new File(path);
        //get all files in the directory et tous les fichier dans les sous dossier
        File[] files = listFiles(f.getAbsolutePath()).stream().map(Path::toFile).toArray(File[]::new);
        if (files == null){
            return "";
        }
        for (File file : files){
            if (file.getName().contains(name)){
                return file.getAbsolutePath();
            }
        }
        return "";
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

    public static void removeOject(Object o, String packname) {
        //look in all file in folder and remove the file
        String path;
        if (packname.contains(".dnxpack")){
            path = Config.getCachePath() + "/pack/" + packname;
        }else {
            path = Config.getLastdirectory() + "/" + packname;
        }
        File f = new File(path);
        //get all files in the directory et tous les fichier dans les sous dossier
        File[] files = listFiles(f.getAbsolutePath()).stream().map(Path::toFile).toArray(File[]::new);
        if (files == null){
            return;
        }
        String name = "";
        String dirObj = "";
        if (o instanceof Block){
            name = "block_" + ((Block) o).getName();
            //get dir obj with model
            dirObj = ((Block) o).getModel();
        }
        if (o instanceof Armor){
            name = "armor_" + ((Armor) o).getName();
            dirObj = ((Armor) o).getModel();
        }
        if (o instanceof Vehicul){
            name = "vehicle_" + ((Vehicul) o).getName();
            dirObj = ((Vehicul) o).getModel();
        }
        if (o instanceof Item){
            name = "item_" + ((Item) o).getName();
            dirObj = ((Item) o).getModel();
        }


    }
}
