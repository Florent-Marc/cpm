package com.mk.cpm.loader.object;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

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
    private String Enchantability;
    private String EquipSound;
    private List<String> infos;
    private Item item;
    private File file;

    @Override
    public Object load(File file) {
        this.file = file;
        this.infos = new ArrayList<>();
        this.item = (Item) super.load(file);
        this.ArmorArms = getValues(infos,file, "ArmorArms");
        this.ArmorBody = getValues(infos,file, "ArmorBody");
        this.ArmorFoot = getValues(infos,file, "ArmorFoot");
        this.ArmorHead = getValues(infos,file, "ArmorHead");
        this.ArmorLegs = getValues(infos,file, "ArmorLegs");
        this.DamageReduction = getValues(infos,file, "DamageReduction");
        this.Durability = getValues(infos,file, "Durability");
        this.Textures = getValues(infos,file, "Textures");
        this.Toughness = getValues(infos,file, "Toughness");
        this.Enchantability = getValues(infos,file, "Enchantability");
        this.EquipSound = getValues(infos,file, "EquipSound");
        this.infos.addAll(super.getInfos());
        return this;
    }

    @Override
    public void save(File file) {
        super.save(file);
        setValues(file, "ArmorArms", ArmorArms);
        setValues(file, "ArmorBody", ArmorBody);
        setValues(file, "ArmorFoot", ArmorFoot);
        setValues(file, "ArmorHead", ArmorHead);
        setValues(file, "ArmorLegs", ArmorLegs);
        setValues(file, "DamageReduction", DamageReduction);
        setValues(file, "Durability", Durability);
        setValues(file, "Textures", Textures);
        setValues(file, "Toughness", Toughness);
        setValues(file, "Enchantability", Enchantability);
        setValues(file, "EquipSound", EquipSound);
        Map<String, String> map = Map.of("Variants", Textures);
        setMultiValues(file, "MaterialVariants",map);
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

    public String getEnchantability() {
        return Enchantability;
    }

    public void setEnchantability(String enchantability) {
        Enchantability = enchantability;
    }

    public String getEquipSound() {
        return EquipSound;
    }

    public void setEquipSound(String equipSound) {
        EquipSound = equipSound;
    }

    public void setInfos(List<String> infos) {
        this.infos = infos;
    }

    public Item getItem() {
        return item;
    }

    public void setItem(Item item) {
        this.item = item;
    }

    public File getFile() {
        return file;
    }

    public void setFile(File file) {
        this.file = file;
    }
}
