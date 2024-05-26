package com.mk.cpm;

import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.input.MouseEvent;
import javafx.stage.Stage;

import java.io.IOException;

public class InfoController {
    @FXML
    public Button back;

    @FXML
    public void back(MouseEvent mouseEvent) {
        //get stage and set close
        Stage stage = (Stage) back.getScene().getWindow();
        stage.close();
    }

    @FXML
    public void discord(MouseEvent mouseEvent) {
        //open discord
        try {
            Runtime.getRuntime().exec("cmd /c start https://discord.gg/7USXZZTf4h");
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
