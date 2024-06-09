package com.mk.cpm.loader.object;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

public class Armor extends Item {

    private String ArmorHead;
    private String ArmorBody;
    private String ArmorArms;
    private String ArmorLegs;
    private String ArmorFoot;

    private String Durability;
    private String Textures;
    private String Toughness;
    private String DamageReduction;
    private List<String> infos;
    private Item item;

    @Override
    public Object load(File file) {
        this.infos = new ArrayList<>();
        this.item = (Item) super.load(file);
        this.infos.addAll(this.item.getInfos());
        this.ArmorArms = getValues(infos,file, "ArmorArms");
        this.ArmorBody = getValues(infos,file, "ArmorBody");
        this.ArmorFoot = getValues(infos,file, "ArmorFoot");
        this.ArmorHead = getValues(infos,file, "ArmorHead");
        this.ArmorLegs = getValues(infos,file, "ArmorLegs");
        this.DamageReduction = getValues(infos,file, "DamageReduction");
        this.Durability = getValues(infos,file, "Durability");
        this.Textures = getValues(infos,file, "Textures");
        this.Toughness = getValues(infos,file, "Toughness");
        return this;
    }

    public Armor() {
    }

    @Override
    public List<String> getInfos() {
        return this.infos;
    }

    public String getArmorHead() {
        return ArmorHead;
    }

    public void setArmorHead(String armorHead) {
        ArmorHead = armorHead;
    }

    public String getArmorBody() {
        return ArmorBody;
    }

    public void setArmorBody(String armorBody) {
        ArmorBody = armorBody;
    }

    public String getArmorArms() {
        return ArmorArms;
    }

    public void setArmorArms(String armorArms) {
        ArmorArms = armorArms;
    }

    public String getArmorLegs() {
        return ArmorLegs;
    }

    public void setArmorLegs(String armorLegs) {
        ArmorLegs = armorLegs;
    }

    public String getArmorFoot() {
        return ArmorFoot;
    }

    public void setArmorFoot(String armorFoot) {
        ArmorFoot = armorFoot;
    }

    public String getDurability() {
        return Durability;
    }

    public void setDurability(String durability) {
        Durability = durability;
    }

    public String getTextures() {
        return Textures;
    }

    public void setTextures(String textures) {
        Textures = textures;
    }

    public String getToughness() {
        return Toughness;
    }

    public void setToughness(String toughness) {
        Toughness = toughness;
    }

    public String getDamageReduction() {
        return DamageReduction;
    }

    public void setDamageReduction(String damageReduction) {
        DamageReduction = damageReduction;
    }
}
