package com.mk.cpm.loader;

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

    public Armor() {
    }

    @Override
    public String[] getInfos() {
        String[] infos = new String[super.getInfos().length + 9];
        infos[0] = "ArmorHead: " + ArmorHead;
        infos[1] = "ArmorBody: " + ArmorBody;
        infos[2] = "ArmorArms: " + ArmorArms;
        infos[3] = "ArmorLegs: " + ArmorLegs;
        infos[4] = "ArmorFoot: " + ArmorFoot;
        infos[5] = "Durability: " + Durability;
        infos[6] = "Textures: " + Textures;
        infos[7] = "Toughness: " + Toughness;
        infos[8] = "DamageReduction: " + DamageReduction;
        String[] superInfos = super.getInfos();
        for (int i = 0; i < superInfos.length; i++) {
            infos[i + 9] = superInfos[i];
        }
        return infos;
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
