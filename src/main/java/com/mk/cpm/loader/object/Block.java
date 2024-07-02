package com.mk.cpm.loader.object;

import com.mk.cpm.loader.utils.DataModifier;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

public class Block extends Item implements DataModifier {

    //item
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
    private List<String> infos;
    private Item item;
    private File file;


    public Block() {
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


    public Item getItem() {
        return item;
    }

    public void setItem(Item item) {
        this.item = item;
    }

    @Override
    public Object load(File file) {
        this.file = file;
        this.item = (Item) super.load(file);
        this.infos = new ArrayList<>();
        this.infos = super.getInfos();
        this.LightLevel = getValues(infos, file, "LightLevel");
        this.Material = getValues(infos, file, "Material");
        this.RenderDistance = getValues(infos, file, "RenderDistance");
        this.Scale = getValues(infos, file, "Scale");
        this.Rotation = getValues(infos, file, "Rotate");
        this.Translation = getValues(infos, file, "Translate");
        this.Textures = getValues(infos, file, "Variants:");
        this.UseComplexCollision = Boolean.parseBoolean(getValues(infos, file, "UseComplexCollision"));
        this.CenterOfGravityOffset = getValues(infos, file, "CenterOfGravityOffset");
        this.EmptyMass = getValues(infos, file, "EmptyMass");
        this.DespawnTime = getValues(infos, file, "DespawnTime");
        return this;
    }

    @Override
    public void save(File file) {
        super.save(file);
        setValues(file, "LightLevel", LightLevel);
        setValues(file, "Material", Material);
        setValues(file, "RenderDistance", RenderDistance);
        setValues(file, "Scale", Scale);
        setValues(file, "Rotate", Rotation);
        setValues(file, "Translate", Translation);
        setValues(file, "Variants:", Textures);
        if (UseComplexCollision != null) {
            setValues(file, "UseComplexCollision", UseComplexCollision.toString());
        }
        setValues(file, "CenterOfGravityOffset", CenterOfGravityOffset);
        setValues(file, "EmptyMass", EmptyMass);
        setValues(file, "DespawnTime", DespawnTime);
    }


    @Override
    public List<String> getInfos() {
        return this.infos;
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


    public String getDespawnTime() {
        return DespawnTime;
    }

    public void setDespawnTime(String despawnTime) {
        DespawnTime = despawnTime;
    }

    public void setInfos(List<String> infos) {
        this.infos = infos;
    }

    public File getFile() {
        return file;
    }

    public void setFile(File file) {
        this.file = file;
    }
}
