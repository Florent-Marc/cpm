package com.mk.cpm.loader;

public class Block {

    //required
    private String name;
    private String desc;
    private String model;
    //item
    private String CreativeTab;
    private String ItemRotation;
    private String ItemTranslation;
    private String ItemScale;
    private String IconText;
    private String Item3DRenderLocation;
    //advanced
    private String LightLevel;
    private String Material;
    private String RenderDistance;
    private String Scale;
    private String Rotation;
    private String Translation;
    private String Textures;
    private Boolean UseComplexCollision;
    //props
    private String CenterOfGravityOffset;
    private String EmptyMass;
    private String DespawnTime;



    public Block() {
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

    public String getScale() {
        return Scale;
    }

    public void setScale(String scale) {
        Scale = scale;
    }

    public String getTranslation() {
        return Translation;
    }

    public void setTranslation(String translation) {
        Translation = translation;
    }

    public String getCreativeTab() {
        return CreativeTab;
    }

    public void setCreativeTab(String creativeTab) {
        CreativeTab = creativeTab;
    }

    public String[] getInfos() {
        return new String[]{"Name : " + name,
                "Description : " + desc,
                "Model : " + model,
                "Scale : " + Scale,
                "Translation : " + Translation,
                "CreativeTab : " + CreativeTab,
                "ItemRotation : " + ItemRotation,
                "ItemTranslation : " + ItemTranslation,
                "ItemScale : " + ItemScale,
                "LightLevel : " + LightLevel,
                "Material : " + Material,
                "RenderDistance : " + RenderDistance,
                "Rotation : " + Rotation,
                "Textures : " + Textures,
                "UseComplexCollision : " + UseComplexCollision,
                "CenterOfGravityOffset : " + CenterOfGravityOffset,
                "EmptyMass : " + EmptyMass,
                "IconText : " + IconText,
                "Item3DRenderLocation : " + Item3DRenderLocation,
                "DespawnTime : " + DespawnTime};
    }

    public String getItemRotation() {
        return ItemRotation;
    }

    public void setItemRotation(String itemRotation) {
        ItemRotation = itemRotation;
    }

    public String getItemTranslation() {
        return ItemTranslation;
    }

    public void setItemTranslation(String itemTranslation) {
        ItemTranslation = itemTranslation;
    }

    public String getItemScale() {
        return ItemScale;
    }

    public void setItemScale(String itemScale) {
        ItemScale = itemScale;
    }

    public String getLightLevel() {
        return LightLevel;
    }

    public void setLightLevel(String lightLevel) {
        LightLevel = lightLevel;
    }

    public String getMaterial() {
        return Material;
    }

    public void setMaterial(String material) {
        Material = material;
    }

    public String getRenderDistance() {
        return RenderDistance;
    }

    public void setRenderDistance(String renderDistance) {
        RenderDistance = renderDistance;
    }

    public String getRotation() {
        return Rotation;
    }

    public void setRotation(String rotation) {
        Rotation = rotation;
    }

    public String getTextures() {
        return Textures;
    }

    public void setTextures(String textures) {
        Textures = textures;
    }

    public Boolean getUseComplexCollision() {
        return UseComplexCollision;
    }

    public void setUseComplexCollision(Boolean useComplexCollision) {
        UseComplexCollision = useComplexCollision;
    }

    public String getCenterOfGravityOffset() {
        return CenterOfGravityOffset;
    }

    public void setCenterOfGravityOffset(String centerOfGravityOffset) {
        CenterOfGravityOffset = centerOfGravityOffset;
    }

    public String getEmptyMass() {
        return EmptyMass;
    }

    public void setEmptyMass(String emptyMass) {
        EmptyMass = emptyMass;
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

    public String getDespawnTime() {
        return DespawnTime;
    }

    public void setDespawnTime(String despawnTime) {
        DespawnTime = despawnTime;
    }
}
