package com.mk.cpm.controller;

import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.LoaderBlock;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.*;
import javafx.scene.input.MouseEvent;
import javafx.stage.Stage;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

public class BlockController implements Initializable {

    private Block block;
    @FXML
    public Tab tabprop;
    public CheckBox isprop;
    public TextField name;
    public TextField desc;
    public TextField model;
    public TextField CreativeTab;
    public ChoiceBox choiceBox;
    public TextField itemrotation;
    public TextField itemtranslation;
    public TextField itemscale;
    public TextField IconText;
    public TextField LightLevel;
    public TextField Material;
    public TextField RenderDistance;
    public TextField Scale;
    public TextField Rotation;
    public TextField Translation;
    public TextField Textures;
    public TextField UseComplexCollision;
    public TextField EmptyMass;
    public TextField CenterOfGravityOffset;
    public TextField RespawnTime;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        if (MainController.blockselected instanceof Block){
             block= (Block) MainController.blockselected;
        }
        if (block == null) {
            Stage stage = (Stage) name.getScene().getWindow();
            stage.close();
        }
        if (block.getEmptyMass() !=null) {
            tabprop.setDisable(false);
            isprop.setSelected(true);
        } else {
            tabprop.setDisable(true);
        }
        if (block.getCreativeTab() != null) {
            CreativeTab.setText(block.getCreativeTab());
        }
        if (block.getModel() != null) {
            model.setText(block.getModel());
        }
        if (block.getDesc() != null) {
            desc.setText(block.getDesc());
        }
        if (block.getName() != null) {
            name.setText(block.getName());
        }
        if (block.getItemRotation() != null) {
            itemrotation.setText(block.getItemRotation());
        }
        if (block.getItemTranslation() != null) {
            itemtranslation.setText(block.getItemTranslation());
        }
        if (block.getItemScale() != null) {
            itemscale.setText(block.getItemScale());
        }
        choiceBox.getItems().addAll("NONE","WORLD","ALL");
        choiceBox.getSelectionModel().selectFirst();
        if (block.getLightLevel() != null) {
            LightLevel.setText(block.getLightLevel());
        }
        if (block.getMaterial() != null) {
            Material.setText(block.getMaterial());
        }
        if (block.getRenderDistance() != null) {
            RenderDistance.setText(block.getRenderDistance());
        }
        if (block.getScale() != null) {
            Scale.setText(block.getScale());
        }
        if (block.getRotation() != null) {
            Rotation.setText(block.getRotation());
        }
        if (block.getTranslation() != null) {
            Translation.setText(block.getTranslation());
        }
        if (block.getTextures() != null) {
            Textures.setText(block.getTextures());
        }
        if (block.getUseComplexCollision() != null) {
            UseComplexCollision.setText(String.valueOf(block.getUseComplexCollision()));
        }
        if (block.getEmptyMass() != null) {
            EmptyMass.setText(block.getEmptyMass());
        }
        if (block.getCenterOfGravityOffset() != null) {
            CenterOfGravityOffset.setText(block.getCenterOfGravityOffset());
        }
        if (block.getDespawnTime() != null) {
            RespawnTime.setText(block.getDespawnTime());
        }
        //icon
        if (block.getIconText() != null) {
            IconText.setText(block.getIconText());
        }

    }

    @FXML
    public void props(MouseEvent mouseEvent) {
        if (isprop.isSelected()) {
            tabprop.setDisable(false);
        } else {
            tabprop.setDisable(true);
        }
    }

    @FXML
    public void save(MouseEvent mouseEvent) {
        File f = LoaderBlock.getFile(block.getName());
        if (f == null) {
            return;
        }
        try {
            FileWriter fw = new FileWriter(f);
            fw.write("Name: " + name.getText() + "\n");
            fw.write("Description: " + desc.getText() + "\n");
            fw.write("Model: " + model.getText() + "\n");
            if(!CreativeTab.getText().isEmpty()){
                fw.write("CreativeTab: " + CreativeTab.getText() + "\n");
            }
            if(!itemrotation.getText().isEmpty()){
                fw.write("ItemRotate: " + itemrotation.getText() + "\n");
            }
            if(!itemtranslation.getText().isEmpty()){
                fw.write("ItemTranslate: " + itemtranslation.getText() + "\n");
            }
            if(!itemscale.getText().isEmpty()){
                fw.write("ItemScale: " + itemscale.getText() + "\n");
            }
            if(!IconText.getText().isEmpty()){
                fw.write("IconText: " + IconText.getText() + "\n");
            }
            if(!LightLevel.getText().isEmpty()){
                fw.write("LightLevel: " + LightLevel.getText() + "\n");
            }
            if(!Material.getText().isEmpty()){
                fw.write("Material: " + Material.getText() + "\n");
            }
            if(!RenderDistance.getText().isEmpty()){
                fw.write("RenderDistance: " + RenderDistance.getText() + "\n");
            }
            if(!Scale.getText().isEmpty()){
                fw.write("Scale: " + Scale.getText() + "\n");
            }
            if(!Rotation.getText().isEmpty()){
                fw.write("Rotate: " + Rotation.getText() + "\n");
            }
            if(!Translation.getText().isEmpty()){
                fw.write("Translate: " + Translation.getText() + "\n");
            }
            if(!Textures.getText().isEmpty()){
                fw.write("Textures: " + Textures.getText() + "\n");
            }
            if(!UseComplexCollision.getText().isEmpty()){
                fw.write("UseComplexCollisions: " + UseComplexCollision.getText() + "\n");
            }
            if(!EmptyMass.getText().isEmpty()){
                String name = block.getName();
                //get first word
                String[] words = name.split(" ");
                fw.write("Prop_"+words[0]+"{" + "\n");
                fw.write("  EmptyMass: " + EmptyMass.getText() + "\n");
                fw.write("  CenterOfGravityOffset: " + CenterOfGravityOffset.getText() + "\n");
                fw.write("  DespawnTime: " + RespawnTime.getText() + "\n");
                fw.write("}" + "\n");
            }

            fw.close();
            Stage stage = (Stage) name.getScene().getWindow();
            stage.close();
            MainController.mainController.refresh();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }
}