package com.mk.cpm.loader.object;

import com.mk.cpm.loader.DataModifier;
import com.mk.cpm.loader.DataUtils;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileWriter;
import java.util.ArrayList;
import java.util.List;

public class Item extends DataUtils implements DataModifier {

    //required
    private String name;
    private String desc;
    private String model;
    //item
    private String CreativeTab;
    private String ItemRotation;
    private String ItemTranslate;
    private String ItemScale;
    private String IconText;
    private String Item3DRenderLocation;
    private String MaxItemStackSize;
    private List<String> infos;

    public Item() {
    }

    public String getMaxItemStackSize() {
        return MaxItemStackSize;
    }

    public void setMaxItemStackSize(String maxItemStackSize) {
        MaxItemStackSize = maxItemStackSize;
    }

    public String getName() {
        return name;
    }

    public String getDesc() {
        return desc;
    }

    public String getModel() {
        return model;
    }

    public void setName(String name) {
        this.name = name;
    }

    public void setDesc(String desc) {
        this.desc = desc;
    }

    public void setModel(String model) {
        this.model = model;
    }


    public String getCreativeTab() {
        return CreativeTab;
    }

    public void setCreativeTab(String creativeTab) {
        CreativeTab = creativeTab;
    }

    public String getItemRotation() {
        return ItemRotation;
    }

    public void setItemRotation(String itemRotation) {
        ItemRotation = itemRotation;
    }

    public String getItemTranslate() {
        return ItemTranslate;
    }

    public void setItemTranslate(String itemTranslate) {
        ItemTranslate = itemTranslate;
    }

    public String getItemScale() {
        return ItemScale;
    }

    public void setItemScale(String itemScale) {
        ItemScale = itemScale;
    }

    public String getIconText() {
        return IconText;
    }

    public void setIconText(String iconText) {
        IconText = iconText;
    }

    public String getItem3DRenderLocation() {
        return Item3DRenderLocation;
    }

    public void setItem3DRenderLocation(String item3DRenderLocation) {
        Item3DRenderLocation = item3DRenderLocation;
    }

    @Override
    public Object load(File file) {
        this.infos = new ArrayList<>();
        this.name = getValues(infos, file, "Name");
        this.desc = getValues(infos, file, "Description");
        this.model = getValues(infos, file, "Model");
        this.CreativeTab = getValues(infos, file, "CreativeTab");
        this.ItemRotation = getValues(infos, file, "ItemRotate");
        this.ItemTranslate = getValues(infos, file, "ItemTranslate");
        this.ItemScale = getValues(infos, file, "ItemScale");
        this.IconText = getValues(infos, file, "IconText");
        this.Item3DRenderLocation = getValues(infos, file, "Item3DRenderLocation");
        this.MaxItemStackSize = getValues(infos, file, "MaxItemStackSize");
        return this;
    }

    @Override
    public void save(FileWriter file) {

    }

    @Override
    public List<String> getInfos() {
        return this.infos;
    }

}
