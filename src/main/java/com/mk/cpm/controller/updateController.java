package com.mk.cpm.controller;

import com.mk.cpm.GitHubReleasesFetcher;
import com.mk.cpm.HelloApplication;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseEvent;

import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

import static com.mk.cpm.GitHubReleasesFetcher.downloadFile;

public class updateController implements Initializable {

    @FXML
    public Label version;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        version.setText(HelloApplication.version+" --> "+ GitHubReleasesFetcher.newVersion);
    }

    @FXML
    public void down(MouseEvent mouseEvent) {
        //close app and download new version and start new version
        String name = GitHubReleasesFetcher.name;
        String path = System.getProperty("user.dir");
        System.out.println("Current path: " + path);
        try {
            downloadFile("https://github.com/Florent-Marc/cpm/releases/download/"+name.replace(" ","-")+"/CPM-UI.jar", path);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        //execute new version
        try {
            Runtime.getRuntime().exec("java -jar "+path+"\\CPM-UI.jar");
        } catch (IOException e) {
            System.out.println("Error while opening discord link");
        }
        System.exit(0);
        System.out.println("Downloaded");
    }
}
