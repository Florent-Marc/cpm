package com.mk.cpm;

import com.mk.cpm.config.Config;
import com.mk.cpm.loader.Block;
import com.mk.cpm.loader.LoaderBlock;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

public class BlockController implements Initializable {
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


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        Block block = MainController.blockselected;
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
        File f = LoaderBlock.getFile(MainController.blockselected.getName());
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
            fw.close();
            Stage stage = (Stage) name.getScene().getWindow();
            stage.close();
            MainController.mainController.refresh();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }
}