package com.mk.cpm.controller;

import com.mk.cpm.AppMain;
import com.mk.cpm.converter.AddonProperties;
import com.mk.cpm.converter.JarLoader;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Priority;
import javafx.stage.FileChooser;
import org.json.simple.JSONObject;
import org.json.simple.parser.JSONParser;
import org.json.simple.parser.ParseException;

import java.io.File;
import java.io.IOException;
import java.io.OutputStream;
import java.net.URL;
import java.nio.file.Files;
import java.util.Objects;
import java.util.ResourceBundle;

public class setting implements Initializable {
    @FXML
    public ListView<HBoxCell> addonlistview;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        File file = new File(System.getenv("APPDATA") + "/cpm/addons/");
        if (!file.exists()) {
            file.mkdirs();
        }
        File[] files = file.listFiles();
        if (files != null) {
            for (File f : files) {
                // parser
                try {
                    JSONObject jsonObject = (JSONObject) new JSONParser().parse(new String(Files.readAllBytes(f.toPath())));
                    // add and precise enabled
                    this.addonlistview.getItems().add(new HBoxCell((String) jsonObject.get("name"), AppMain.config.getAddonSettingConfig().getEnabledAddons().contains(jsonObject.get("name"))));
                } catch (ParseException | IOException e) {
                    e.printStackTrace();
                }
            }
        }
    }

    @FXML
    public void folderaddon(MouseEvent mouseEvent) {
        File file = new File(System.getenv("APPDATA") + "/cpm/addons/");
        if (!file.exists()) {
            file.mkdirs();
        }
        try {
            Runtime.getRuntime().exec("explorer.exe " + file.getAbsolutePath());
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public static class HBoxCell extends HBox {
        Label label = new Label();
        Button button = new Button();
        boolean enabled;
        String name;

        HBoxCell(String labelText, boolean enabled) {
            super();
            this.enabled = enabled;
            this.name = labelText;

            label.setText(labelText);
            label.setMaxWidth(Double.MAX_VALUE);
            HBox.setHgrow(label, Priority.ALWAYS);

            button.setText(enabled ? "Disable" : "Enable");
            button.setOnAction(new EventHandler<ActionEvent>() {
                @Override
                public void handle(ActionEvent event) {
                    if(HBoxCell.this.enabled) {
                        AppMain.config.getAddonSettingConfig().removeEnabledAddon(HBoxCell.this.name);
                        button.setText("Enable");
                    } else {
                        AppMain.config.getAddonSettingConfig().addEnabledAddon(HBoxCell.this.name);
                        button.setText("Disable");
                    }
                    AppMain.config.saveConfig();
                    HBoxCell.this.enabled = !HBoxCell.this.enabled;
                }
            });

            this.getChildren().addAll(label, button);
        }
    }


    @FXML
    public void addonconvert(MouseEvent mouseEvent) {
        FileChooser fileChooser = new FileChooser();
        fileChooser.getExtensionFilters().add(new FileChooser.ExtensionFilter("Select your DynamX Addon", "*.jar"));
        System.out.println(AppMain.config.getAddonSettingConfig().getLastFolder());
        if (AppMain.config.getAddonSettingConfig().getLastFolder() != null && !Objects.equals(AppMain.config.getAddonSettingConfig().getLastFolder(), ""))
            fileChooser.setInitialDirectory(new File(AppMain.config.getAddonSettingConfig().getLastFolder()));
        File file = fileChooser.showOpenDialog(null);
        String added = file.getName();
        AddonProperties addonProperties = JarLoader.loadJar("C:\\Users\\gabid\\Desktop\\cpm\\lib\\DynamX-4.1.0-dev19-all-deobf.jar", file.getAbsolutePath());

        File addons = new File(System.getenv("APPDATA") + "/cpm/addons/");
        if (!addons.exists()) {
            addons.mkdirs();
        }
        System.out.println(addonProperties.toJson());
        File addon = new File(addons, addonProperties.name + ".json");

        try {
            Files.write(addon.toPath(), addonProperties.toJson().getBytes());
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        this.addonlistview.getItems().add(new HBoxCell(addonProperties.name, true));


        AppMain.config.getAddonSettingConfig().addEnabledAddon(addonProperties.name);
        AppMain.config.getAddonSettingConfig().setLastFolder(file.getParent());
        AppMain.config.saveConfig();
    }


}
