package com.mk.cpm;

import com.mk.cpm.config.Config;
import imgui.ImGui;
import imgui.app.Configuration;
import javafx.animation.AnimationTimer;
import javafx.application.Application;
import javafx.fxml.FXMLLoader;
import javafx.scene.Group;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.canvas.Canvas;
import javafx.scene.control.ContentDisplay;
import javafx.scene.image.Image;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;

import java.io.IOException;

public class HelloApplication extends Application{
    public static Scene f;
    public static Image logo= new Image(HelloApplication.class.getResourceAsStream("logo.png"));
    public static double version = 0.8;

    @Override
    public void start(Stage stage) throws IOException {
        // Initialize ImGui context
        ImGui.createContext();
        // Set up OpenGL context
        setupOpenGLContext();
        Canvas canvas = new Canvas(800, 600);
        Group root = new Group(canvas);
        Stage s = new Stage();
        FXMLLoader gf = new FXMLLoader(HelloApplication.class.getResource("test.fxml"));
        Parent root1 = gf.load();
        if (root1 instanceof Pane) {
            ((Pane) root1).getChildren().add(root);
        }
        Scene sc = new Scene(root1);


        s.getIcons().add(HelloApplication.logo);
        s.initStyle(StageStyle.UNDECORATED);
        s.setTitle("CPM-UI");
        s.setScene(sc);
        s.setResizable(true);
        s.show();
        sc.lookup("#anchor").setStyle("-fx-background-color: #141b35;");
        sc.lookup("#anchor").lookup("#divider").setStyle("-fx-background-color: #282c34;");
        f = sc;
        if (AppMain.config.isWarning()) {
            FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("warning.fxml"));
            Scene scene = new Scene(fxmlLoader.load());
            stage.getIcons().add(logo);
            scene.setFill(Color.rgb(40, 44, 52, 1.0));
            stage.setTitle("CPM-UI");
            stage.setScene(scene);
            stage.setResizable(false);
            stage.show();
        }
        //login 270 342
        if(AppMain.config.getnewRequest()<System.currentTimeMillis()) {
            // plus 2 minutes
            AppMain.config.setnewRequest(System.currentTimeMillis() + 120000);
            AppMain.config.saveConfig();
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

        new AnimationTimer() {
            @Override
            public void handle(long now) {
                renderImGui();
                // JavaFX rendering logic
            }
        }.start();


    }


    public static void main(String[] args) {
        launch();
    }

    private void setupOpenGLContext() {
        // Initialize OpenGL context, link with JavaFX
    }

    private void renderImGui() {
        // ImGui render logic


        ImGui.getIO().setDisplaySize(800, 600);
        ImGui.newFrame();
        ImGui.text("Hello, ImGui in JavaFX!");
        ImGui.render();
    }

}
