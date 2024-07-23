package com.mk.cpm.controller;

import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.input.MouseEvent;

import java.io.File;
import java.net.URL;
import java.util.ResourceBundle;

public class setting implements Initializable {
    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {

    }

    @FXML
    public void folderaddon(MouseEvent mouseEvent) {
        File file = new File(System.getenv("APPDATA") + "/cpm/addons/");
        if (!file.exists()) {
            file.mkdirs();
        }
        //open the folder
        try {
            Runtime.getRuntime().exec("explorer.exe " + file.getAbsolutePath());
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
