package com.mk.cpm;

import com.mk.cpm.loader.Block;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;

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


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        if (isprop.isSelected()) {
            tabprop.setDisable(false);
        } else {
            tabprop.setDisable(true);
        }
        Block block = MainController.blockselected;
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
}