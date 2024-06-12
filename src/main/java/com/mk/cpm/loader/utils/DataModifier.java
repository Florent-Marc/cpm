package com.mk.cpm.loader.utils;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileWriter;
import java.util.List;

public interface DataModifier {
    Object load(File file);
    void save(FileWriter file);

    List<String> getInfos();
}
