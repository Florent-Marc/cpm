package com.mk.cpm.loader.utils;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileWriter;
import java.util.List;

public interface DataModifier {
    Object load(File file);

    default void save(File file) {
    }

    List<String> getInfos();
}
