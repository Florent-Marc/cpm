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
    public static Scene f;
    public static Image logo= new Image(HelloApplication.class.getResourceAsStream("logo.png"));

    @Override
    public void start(Stage stage) throws IOException {
        Stage s = new Stage();
        FXMLLoader gf = new FXMLLoader(HelloApplication.class.getResource("test.fxml"));
        Scene sc = new Scene(gf.load());
        s.getIcons().add(HelloApplication.logo);
        s.setTitle("CPM-UI");
        s.setScene(sc);
        s.setResizable(true);
        s.show();
        f = sc;
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("warning.fxml"));
        Scene scene = new Scene(fxmlLoader.load());
        stage.getIcons().add(logo);
        scene.setFill(Color.rgb(40, 44, 52, 1.0));
        stage.setTitle("CPM-UI");
        stage.setScene(scene);
        stage.setResizable(false);
        stage.show();
        //login 270 342
    }


    public static void main(String[] args) {
        launch();

    }

}
