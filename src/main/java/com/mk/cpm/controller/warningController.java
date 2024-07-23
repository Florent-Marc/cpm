package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;

import java.net.URL;
import java.util.ResourceBundle;

public class warningController implements Initializable {
    @FXML
    public Label vr;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        vr.setText("Version "+HelloApplication.version);
        vr.setAlignment(javafx.geometry.Pos.CENTER);
    }
}
