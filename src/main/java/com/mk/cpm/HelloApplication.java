package com.mk.cpm;

import javafx.application.Application;
import javafx.fxml.FXMLLoader;
import javafx.scene.Scene;
import javafx.scene.control.ContentDisplay;
import javafx.scene.image.Image;
import javafx.scene.paint.Color;
import javafx.stage.Stage;

import java.io.IOException;

public class HelloApplication extends Application {
    public static Scene s;
    public static Image logo= new Image(HelloApplication.class.getResourceAsStream("logo.png"));

    @Override
    public void start(Stage stage) throws IOException {
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("login.fxml"));
        Scene scene = new Scene(fxmlLoader.load(), 270, 342);
        stage.getIcons().add(logo);
        scene.setFill(Color.rgb(40, 44, 52, 1.0));
        stage.setTitle("CPM-UI");
        stage.setScene(scene);
        stage.setResizable(false);
        stage.show();
        s = scene;
        //login 270 342
    }


    public static void main(String[] args) {
        launch();

    }

}
