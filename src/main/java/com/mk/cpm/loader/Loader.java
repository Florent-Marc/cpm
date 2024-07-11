package com.mk.cpm.loader;

import com.mk.cpm.Main;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.object.*;
import com.mk.cpm.loader.pack.ZipExtractor;

import java.io.File;
import java.io.IOException;
import java.nio.file.*;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

public class Loader {

    private static List<Object> objectsList;

    public static List<Object> getBlocksByPack(String packName) {
        if (packName == null) {
            return new ArrayList<>();
        }
        List<Object> objects = new ArrayList<>();
        File file = prepareFile(packName);
        String[] bannedNames = {"assets", "pack_info.dynx"};
        File[] files = listFiles(file.getAbsolutePath()).stream().map(Path::toFile).toArray(File[]::new);
        for (File f : files) {
            if (isBanned(f.getName(), bannedNames)) continue;
            loadObject(f, objects);
        }
        objectsList = objects;
        return objects;
    }

    private static File prepareFile(String packName) {
        File file;
        if (Main.isDynxPack) {
            try {
                ZipExtractor.extractZip(Config.getLastdirectory() + "/" + packName, Config.getCachePath() + "/pack/" + packName);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
            file = new File(Config.getCachePath() + "/pack/" + packName);
        } else {
            file = new File(Config.getLastdirectory() + "/" + packName);
        }
        return file;
    }

    private static boolean isBanned(String fileName, String[] bannedNames) {
        for (String ban : bannedNames) {
            if (fileName.contains(ban)) return true;
        }
        return false;
    }

    private static void loadObject(File file, List<Object> objects) {
        String fileName = file.getName();
        if (fileName.contains("block_")) {
            Block block = new Block();
            block.load(file);
            objects.add(block);
        } else if (fileName.contains("armor_")) {
            Armor armor = new Armor();
            armor.load(file);
            objects.add(armor);
        } else if (fileName.contains("vehicle_")) {
            Vehicul vehicul = new Vehicul();
            vehicul.load(file);
            objects.add(vehicul);
        } else if (fileName.contains("item_")) {
            Item item = new Item();
            item.load(file);
            objects.add(item);
        }
    }

    public static String getPath(String name, String packName) {
        String path = packName.contains(".dnxpack") ? Config.getCachePath() + "/pack/" + packName : Config.getLastdirectory() + "/" + packName;
        File[] files = listFiles(path).stream().map(Path::toFile).toArray(File[]::new);
        for (File file : files) {
            if (file.getName().contains(name)) {
                return file.getAbsolutePath();
            }
        }
        return "";
    }

    public static List<Path> listFiles(String directory) {
        try (Stream<Path> paths = Files.walk(Paths.get(directory))) {
            return paths.filter(Files::isRegularFile).collect(Collectors.toList());
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    public static List<Object> getObject() {
        return objectsList;
    }

    public static void removeObject(Object o, String packName) {
        String path = packName.contains(".dnxpack") ? Config.getCachePath() + "/pack/" + packName : Config.getLastdirectory() + "/" + packName;
        File[] files = listFiles(path).stream().map(Path::toFile).toArray(File[]::new);
        String name = getName(o);
        if (name.isEmpty()) return;
        for (File file : files) {
            if (file.getName().contains(name)) {
                file.deleteOnExit();
                break;
            }
        }
    }

    private static String getName(Object o) {
        if (o instanceof Block) {
            return "block_" + ((Block) o).getName();
        } else if (o instanceof Armor) {
            return "armor_" + ((Armor) o).getName();
        } else if (o instanceof Vehicul) {
            return "vehicle_" + ((Vehicul) o).getName();
        } else if (o instanceof Item) {
            return "item_" + ((Item) o).getName();
        }
        return "";
    }
}