package com.mk.cpm.loader.object;

import com.mk.cpm.loader.utils.DataModifier;
import com.mk.cpm.loader.utils.DataUtils;

import java.io.File;
import java.io.FileWriter;
import java.util.ArrayList;
import java.util.List;

public class Shape extends DataUtils implements DataModifier {

    private String Position;
    private String Scale;
    private String Name;
    private List<String> infos;

    @Override
    public Object load(File file) {
        return null;
    }

    public Object load(File file, String name) {
        this.infos = new ArrayList<>();
        this.Name = name;
        this.Position = getValuesFromSection(infos,file, name, "Position");
        this.Scale = getValuesFromSection(infos,file, name, "Scale");
        return this;
    }

    public String getName() {
        return Name;
    }

    public void setName(String name) {
        Name = name;
    }

    public String getPosition() {
        return Position;
    }

    public void setPosition(String position) {
        Position = position;
    }

    public String getScale() {
        return Scale;
    }

    public void setScale(String scale) {
        Scale = scale;
    }

    public void setInfos(List<String> infos) {
        this.infos = infos;
    }

    @Override
    public void save(File file) {

    }

    @Override
    public List<String> getInfos() {
        return this.infos;
    }
}
