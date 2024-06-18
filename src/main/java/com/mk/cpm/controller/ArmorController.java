package com.mk.cpm.controller;

import com.mk.cpm.config.Config;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.pack.ZipCompressor;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ChoiceBox;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.stage.Stage;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

public class ArmorController implements Initializable {

    private Armor armor;
    @FXML
    public ChoiceBox<Object> choiceBox;
    public TextField name;
    public TextField desc;
    public TextField model;
    public TextField CreativeTab;
    public TextField itemrotation;
    public TextField itemtranslation;
    public TextField itemscale;
    public TextField IconText;
    public TextField ArmorArms;
    public TextField ArmorBody;
    public TextField ArmorFoot;
    public TextField ArmorHead;
    public TextField ArmorLegs;
    public TextField DamageReduction;
    public TextField Durability;
    public TextField Textures;
    public TextField Toughness;
    public TextField Enchantability;
    public TextField EquipSound;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        if (MainController.blockselected == null) {
            return;
        }
        armor = (Armor) MainController.blockselected;
        if (armor == null) {
            return;
        }
        if (armor.getCreativeTab() != null) {
            CreativeTab.setText(armor.getCreativeTab());
        }
        if (armor.getModel() != null) {
            model.setText(armor.getModel());
        }
        if (armor.getDesc() != null) {
            desc.setText(armor.getDesc());
        }
        if (armor.getName() != null) {
            name.setText(armor.getName());
        }
        if (armor.getItemRotation() != null) {
            itemrotation.setText(armor.getItemRotation());
        }
        if (armor.getItem().getItemTranslate() != null) {
            itemtranslation.setText(armor.getItem().getItemTranslate());
        }
        if (armor.getItemScale() != null) {
            itemscale.setText(armor.getItemScale());
        }
        choiceBox.getItems().addAll("NONE", "WORLD", "ALL");
        choiceBox.getSelectionModel().selectFirst();
        if (armor.getIconText() != null) {
            IconText.setText(armor.getIconText());
        }
        if (armor.getArmorArms() != null) {
            ArmorArms.setText(armor.getArmorArms());
        }
        if (armor.getArmorBody() != null) {
            ArmorBody.setText(armor.getArmorBody());
        }
        if (armor.getArmorFoot() != null) {
            ArmorFoot.setText(armor.getArmorFoot());
        }
        if (armor.getArmorHead() != null) {
            ArmorHead.setText(armor.getArmorHead());
        }
        if (armor.getArmorLegs() != null) {
            ArmorLegs.setText(armor.getArmorLegs());
        }
        if (armor.getDamageReduction() != null) {
            DamageReduction.setText(armor.getDamageReduction());
        }
        if (armor.getDurability() != null) {
            Durability.setText(armor.getDurability());
        }
        if (armor.getTextures() != null) {
            Textures.setText(armor.getTextures());
        }
        if (armor.getToughness() != null) {
            Toughness.setText(armor.getToughness());
        }
        if (armor.getEnchantability() != null) {
            Enchantability.setText(armor.getEnchantability());
        }
        if (armor.getEquipSound() != null) {
            EquipSound.setText(armor.getEquipSound());
        }
    }

    @FXML
    public void save(MouseEvent mouseEvent) {
        if (armor == null) {
            return;
        }
        armor.setCreativeTab(CreativeTab.getText());
        armor.setModel(model.getText());
        armor.setDesc(desc.getText());
        armor.setName(name.getText());
        armor.setItemRotation(itemrotation.getText());
        armor.getItem().setItemTranslate(itemtranslation.getText());
        armor.setItemScale(itemscale.getText());
        armor.setIconText(IconText.getText());
        armor.setArmorArms(ArmorArms.getText());
        armor.setArmorBody(ArmorBody.getText());
        armor.setArmorFoot(ArmorFoot.getText());
        armor.setArmorHead(ArmorHead.getText());
        armor.setArmorLegs(ArmorLegs.getText());
        armor.setDamageReduction(DamageReduction.getText());
        armor.setDurability(Durability.getText());
        armor.setTextures(Textures.getText());
        armor.setToughness(Toughness.getText());
        armor.setEnchantability(Enchantability.getText());
        armor.setEquipSound(EquipSound.getText());
        File f = armor.getFile();
        if (f == null) {
            return;
        }

        try {
            FileWriter fileWriter = new FileWriter(f);
            fileWriter.write("");
            fileWriter.close();
        } catch (Exception e) {
            e.printStackTrace();
        }
        this.armor.save(f);
        if (f.getPath().contains("\\cpm\\cache")) {
            try {
                String source = Config.getCachePath() + "/pack/" + MainController.packname;
                String dest = Config.getLastdirectory() + "/" + MainController.packname;
                ZipCompressor.compressFolder(source, dest);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        }
        Stage stage = (Stage) name.getScene().getWindow();
        stage.close();
        MainController.mainController.refresh();
    }
}
