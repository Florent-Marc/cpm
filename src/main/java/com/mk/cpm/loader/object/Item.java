package com.mk.cpm.loader.object;

public class Item {

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

    public String[] getInfos() {
        return new String[]{"Name : " + name,
            "Description : " + desc,
            "Model : " + model,
            "CreativeTab : " + CreativeTab,
            "ItemRotation : " + ItemRotation,
            "ItemTranslation : " + ItemTranslate,
            "ItemScale : " + ItemScale,
            "IconText : " + IconText,
            "Item3DRenderLocation : " + Item3DRenderLocation,
            "MaxItemStackSize : " + MaxItemStackSize
        };
    }

}
