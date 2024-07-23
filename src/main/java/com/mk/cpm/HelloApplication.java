package com.mk.cpm;

import com.mk.cpm.config.Config;
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
    public static double version = 0.8;

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
        if(Config.getnewRequest()<System.currentTimeMillis()) {
            // plus 2 minutes
            Config.setnewRequest(System.currentTimeMillis() + 120000);
            Config.saveConfig();
            if (GitHubReleasesFetcher.isNewVersionAvailable(version)) {
                Stage fd = new Stage();
                FXMLLoader jk = new FXMLLoader(HelloApplication.class.getResource("update.fxml"));
                Scene d = new Scene(jk.load());
                fd.getIcons().add(logo);
                fd.setTitle("CPM-UI");
                fd.setScene(d);
                fd.setResizable(false);
                fd.show();
            }
        }else {
            System.out.println("check later");
        }

    }


    public static void main(String[] args) {
        launch();
    }

}
