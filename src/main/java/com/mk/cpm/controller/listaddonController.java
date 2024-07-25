package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import com.mk.cpm.converter.AddonProperties;
import com.mk.cpm.loader.MultiParam;
import com.mk.cpm.loader.object.Vehicle;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.*;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;
import javafx.scene.text.Text;
import org.json.simple.JSONObject;
import org.json.simple.parser.JSONParser;
import org.json.simple.parser.ParseException;

import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.StandardOpenOption;
import java.util.*;

public class listaddonController implements Initializable {
    public AddonProperties.TYPES type;
    @FXML
    public VBox vbox1;
    public ScrollPane scrollpane1;
    public Vehicle vehicle;

    public HashMap<String, HashMap<String, String>> newProperties = new HashMap<>();
    public Button save;
    public Button cancel;

    public listaddonController(AddonProperties.TYPES type, Vehicle vehicle) {
        this.type = type;
        this.vehicle = vehicle;

    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        vbox1.getChildren().clear();
        vbox1.setSpacing(5);
        System.out.println(vehicle.otherProperties);
        scrollpane1.setContent(vbox1);
        scrollpane1.setFitToWidth(true);
        scrollpane1.setFitToHeight(true);
        scrollpane1.minWidthProperty().bind(vbox1.minWidthProperty());
        scrollpane1.minHeightProperty().bind(vbox1.minHeightProperty());
        scrollpane1.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollpane1.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        vbox1.prefWidthProperty().bind(scrollpane1.widthProperty());

        try {
            List<AddonProperties> addonProperties = getAllEnabledAddons();

            List<Node> nodes = new ArrayList<>();

            vehicle.otherProperties.forEach((k, v) -> {
                try {
                    AddonProperties relatedAddon = null;
                    for (AddonProperties addonProperty : addonProperties) {
                        if (addonProperty.vehiclesProperties == null) continue;
                        if (addonProperty.vehiclesProperties.containsKey(k.split("_")[0])) {
                            relatedAddon = addonProperty;
                            break;
                        }
                    }
                    System.out.println("Related addon: " + relatedAddon);
                    Node node = FXMLLoader.load(Objects.requireNonNull(HelloApplication.class.getResource("param.fxml")));
                    ((Label) node.lookup("#paramname")).setText(k + " from ");
                    if (relatedAddon != null) {
                        ((Label) node.lookup("#paramname")).setText(k + " from " + relatedAddon.name);
                    } else {
                        return;
                    }

                    VBox vbox = (VBox) ((ScrollPane) node.lookup("#scrollpane2")).getContent().lookup("#vbox2");
                    vbox.setSpacing(5);
                    newProperties.put(k, new HashMap<>());
                    v.forEach((k1, v1) -> {
                        newProperties.get(k).put(k1, v1);
                        HBox hBox = new HBox();
                        hBox.setSpacing(5);
                        Text label = new Text(k1);
                        label.setFill(javafx.scene.paint.Color.WHITE);
                        TextField textField = new TextField(v1);
                        textField.setPromptText("Enter value");
                        textField.setMinWidth(200);
                        textField.setMaxWidth(200);
                        textField.setPrefWidth(200);
                        textField.setOnKeyTyped(event -> {
                            newProperties.get(k).replace(k1, textField.getText());
                        });
                        label.setStyle("-fx-font-weight: bold; -fx-text-fill: #FFFFFF;");
                        hBox.getChildren().add(label);
                        hBox.getChildren().add(textField);
                        vbox.getChildren().add(hBox);

                    });
                    ((ScrollPane) node.lookup("#scrollpane2")).setContent(vbox);
                    ((ScrollPane) node.lookup("#scrollpane2")).setContent(vbox);

                    nodes.add(node);
                } catch (IOException e) {
                    throw new RuntimeException(e);
                }
            });

            HBox hBox = new HBox();
            hBox.setSpacing(5);
            Text label = new Text("Add new property");
            label.setFill(javafx.scene.paint.Color.WHITE);
            ComboBox<ComboBoxElement> comboBox = new ComboBox<>();


            for (AddonProperties addonProperty : addonProperties) {
                if (type == AddonProperties.TYPES.WHEDDLED_VEHICLES) {
                    if (addonProperty.vehiclesProperties != null) {
                        addonProperty.vehiclesProperties.forEach((k, v) -> {
                            comboBox.getItems().add(new ComboBoxElement(addonProperty, k, v));
                        });
                    }
                }
            }

            TextField customName = new TextField();
            customName.setPromptText("Custom name");
            customName.setMinWidth(200);
            customName.setMaxWidth(200);

            Button button = new Button("Add");
            button.setOnAction(event -> {
                if (comboBox.getValue() == null) return;
                if (customName.getText().isEmpty()) return;
                Node node = null;
                try {
                    node = FXMLLoader.load(Objects.requireNonNull(HelloApplication.class.getResource("param.fxml")));
                } catch (IOException e) {
                    throw new RuntimeException(e);
                }
                ((Label) node.lookup("#paramname")).setText(comboBox.getValue().name + "_" + customName.getText() + " from " + comboBox.getValue().addonProperties.name);
                VBox vbox = (VBox) ((ScrollPane) node.lookup("#scrollpane2")).getContent().lookup("#vbox2");
                vbox.setSpacing(5);
                comboBox.getValue().properties.forEach(s -> {
                    HBox hBox1 = new HBox();
                    hBox1.setSpacing(5);
                    Text label1 = new Text(s);
                    label1.setFill(javafx.scene.paint.Color.WHITE);
                    TextField textField = new TextField();
                    textField.setPromptText("Enter value");
                    textField.setMinWidth(200);
                    textField.setMaxWidth(200);
                    textField.setPrefWidth(200);
                    hBox1.getChildren().add(label1);
                    hBox1.getChildren().add(textField);
                    vbox.getChildren().add(hBox1);
                });
                ((ScrollPane) node.lookup("#scrollpane2")).setContent(vbox);

                vbox1.getChildren().add(node);
            });

            hBox.getChildren().add(label);
            hBox.getChildren().add(comboBox);
            hBox.getChildren().add(customName);
            hBox.getChildren().add(button);


            vbox1.getChildren().add(hBox);
            vbox1.getChildren().addAll(nodes);


        } catch (IOException | ParseException e) {
            throw new RuntimeException(e);
        }

        cancel.setOnAction(event -> {
            this.cancel.getScene().getWindow().hide();
        });

        save.setOnAction(event -> {
            newProperties.forEach((k, v) -> {
                vehicle.otherProperties.remove(k);

                vehicle.otherProperties.put(k, v);
            });

            File file = new File(System.getenv("APPDATA") + "/cpm/vehicles/");
            if (!file.exists()) {
                file.mkdirs();
            }
            try {
                Files.deleteIfExists(vehicle.getFile().toPath());
                Files.createFile(vehicle.getFile().toPath());
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
            vehicle.save(vehicle.getFile());
            this.save.getScene().getWindow().hide();
        });
    }

    public static class ComboBoxElement {
        public AddonProperties addonProperties;
        public String name;
        public List<String> properties;

        public ComboBoxElement(AddonProperties addonProperties, String name, List<String> properties) {
            this.addonProperties = addonProperties;
            this.name = name;
            this.properties = properties;
        }

        @Override
        public String toString() {
            return name;
        }
    }

    public List<AddonProperties> getAllEnabledAddons() throws IOException, ParseException {
        File file = new File(System.getenv("APPDATA") + "/cpm/addons/");
        List<AddonProperties> list = new ArrayList<>();

        for (File f : Objects.requireNonNull(file.listFiles())) {
            JSONObject jsonObject = (JSONObject) new JSONParser().parse(new String(Files.readAllBytes(f.toPath())));
            AddonProperties addonProperties = AddonProperties.fromJson(jsonObject);
            list.add(addonProperties);
        }
        return list;
    }



}
