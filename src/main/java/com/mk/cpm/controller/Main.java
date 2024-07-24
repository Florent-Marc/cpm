package com.mk.cpm.controller;

import com.mk.cpm.AppMain;
import com.mk.cpm.HelloApplication;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.Loader;
import com.mk.cpm.loader.LoaderPack;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.object.Item;
import com.mk.cpm.loader.object.Vehicul;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.control.ScrollPane;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;
import javafx.stage.DirectoryChooser;
import javafx.stage.Stage;

import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.ResourceBundle;

public class Main implements Initializable {

    public static String packSelected;
    public static Object objectSelect;
    public static Main Instance;
    @FXML
    public VBox PackList, vbox2, vbox3;
    public ScrollPane scrollPane, scrollpane2, scrollpane3;
    public ImageView reload;
    public ImageView add;
    public ImageView filter;
    public ImageView packadd;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        Instance = this;
        refresh();
    }

    public void LoadPack(String name) {
        packSelected = name;
        AppMain.isDynxPack = name.contains(".dnxpack");
        List<Object> objects = new ArrayList<>();
        objects = Loader.getBlocksByPack(name);
        VBox vBox2 = vbox2;
        vBox2.getChildren().clear();
        vbox3.getChildren().clear();
        vBox2.setSpacing(5);

        scrollpane2.setContent(vBox2);
        scrollpane2.setFitToWidth(true);
        scrollpane2.setFitToHeight(true);
        scrollpane2.minWidthProperty().bind(vBox2.minWidthProperty());
        scrollpane2.minHeightProperty().bind(vBox2.minHeightProperty());
        scrollpane2.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollpane2.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        vBox2.prefWidthProperty().bind(scrollpane2.widthProperty());

        String imagePath = "/com/mk/cpm/texture/";
        Image car = new Image(getClass().getResourceAsStream(imagePath + "car.png"));
        Image block = new Image(getClass().getResourceAsStream(imagePath + "cube.png"));
        Image armor = new Image(getClass().getResourceAsStream(imagePath + "armor.png"));
        Image item = new Image(getClass().getResourceAsStream(imagePath + "axe.png"));
        Node[] nodes2 = new Node[objects.size()];
        for (int i = 0; i < objects.size(); i++) {
            try {
                final int j = i;
                nodes2[i] = FXMLLoader.load(HelloApplication.class.getResource("itemcontent.fxml"));
                Object o = objects.get(i);
                String text1 = "";
                String text2 = "";
                String text3 = "";
                Image img = null;
                List<String> l = new ArrayList<>();
                if (objects.get(i) instanceof Block) {
                    text1 = ((Block) o).getName();
                    if (((Block) o).getEmptyMass() == null) {
                        text2 = "Is not a props";
                    } else {
                        text2 = "Is a props";
                    }
                    text3 = ((Block) o).getModel();
                    img = block;
                    l = ((Block) o).getInfos();
                } else if (objects.get(i) instanceof Armor) {
                    text1 = ((Armor) o).getName();
                    text2 = "Armor";
                    text3 = ((Armor) o).getModel();
                    img = armor;
                    l = ((Armor) o).getInfos();
                } else if (o instanceof Vehicul) {
                    text1 = ((Vehicul) o).getName();
                    text2 = "Vehicul";
                    text3 = ((Vehicul) o).getModel();
                    img = car;
                    l = ((Vehicul) o).getInfos();
                } else if (o instanceof Item) {
                    text1 = ((Item) o).getName();
                    text2 = "Item";
                    text3 = ((Item) o).getModel();
                    img = item;
                    l = ((Item) o).getInfos();
                }
                final List<String> f = l;
                final Object fo = o;
                ((Label) nodes2[i].lookup("#un")).setText(text1);
                ((Label) nodes2[i].lookup("#deux")).setText(text2);
                ((Label) nodes2[i].lookup("#trois")).setText(text3);
                ((ImageView) nodes2[i].lookup("#img")).setImage(img);
                vBox2.getChildren().add(nodes2[i]);
                ((AnchorPane) nodes2[j].lookup("#box")).setOnMouseEntered(event -> {
                    ((AnchorPane) nodes2[j].lookup("#box")).setStyle("-fx-background-color: #007AFF");
                });
                ((AnchorPane) nodes2[j].lookup("#box")).setOnMouseExited(event -> {
                    ((AnchorPane) nodes2[j].lookup("#box")).setStyle("-fx-background-color:  #141b35");
                });
                ((ImageView) nodes2[i].lookup("#edit")).setOnMouseClicked(event -> {
                    objectSelect = fo;
                    loadStage(fo.getClass().getSimpleName().toLowerCase());
                    event.consume();
                });
                ((ImageView) nodes2[i].lookup("#delete")).setOnMouseClicked(event -> {
                    Loader.removeObject(fo, packSelected);
                    event.consume();
                });
                ((ImageView) nodes2[i].lookup("#duplicate")).setOnMouseClicked(event -> {
                    System.out.println("duplicate " + j);
                    event.consume();
                });
                ((AnchorPane) nodes2[j].lookup("#box")).setOnMouseClicked(event -> {
                    LoadInfo(f);
                });
            } catch (IOException e) {
                e.printStackTrace();
            }
        }
        if (packSelected == null || packSelected.isEmpty()) {
            add.setVisible(false);
            filter.setVisible(false);
        }else {
            add.setVisible(true);
            filter.setVisible(true);
        }
    }

    private Stage loadStage(String type) {
        Stage stage = new Stage();
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource(type + "Modifier.fxml"));
        try {
            Scene scene = new Scene(fxmlLoader.load());
            stage.setTitle("CPM-UI (InProgress)");
            stage.setResizable(false);
            stage.getIcons().add(HelloApplication.logo);
            stage.setScene(scene);
            stage.show();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        return stage;
    }

    public void LoadInfo(List<String> l) {
        VBox vBox3 = vbox3;
        vBox3.getChildren().clear();
        vBox3.setSpacing(2);

        scrollpane3.setContent(vBox3);
        scrollpane3.setFitToWidth(true);
        scrollpane3.setFitToHeight(true);
        scrollpane3.minWidthProperty().bind(vBox3.minWidthProperty());
        scrollpane3.minHeightProperty().bind(vBox3.minHeightProperty());
        scrollpane3.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollpane3.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);

        Node[] nodes3 = new Node[l.size()];
        for (int i = 0; i < l.size(); i++) {
            try {
                nodes3[i] = FXMLLoader.load(HelloApplication.class.getResource("iteminfo.fxml"));
                ((Label) nodes3[i].lookup("#info")).setText(l.get(i));
                vBox3.getChildren().add(nodes3[i]);
            } catch (IOException e) {
                e.printStackTrace();
            }
        }
    }

    //refresh the list of packs
    public void refresh() {
        packSelected = null;
        PackList.getChildren().clear();
        vbox3.getChildren().clear();
        vbox2.getChildren().clear();
        String imagePath = "/com/mk/cpm/texture/";
        VBox vBox = PackList;
        vBox.setSpacing(5);

        scrollPane.setContent(vBox);
        scrollPane.setFitToWidth(true); // Ajuste le contenu en largeur
        scrollPane.setFitToHeight(true); // Ajuste le contenu en hauteur

        scrollPane.minWidthProperty().bind(vBox.minWidthProperty());
        scrollPane.minHeightProperty().bind(vBox.minHeightProperty());
        scrollPane.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollPane.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollPane.setHvalue(1);


        Image dnx = new Image(getClass().getResourceAsStream(imagePath + "dnx.png"));
        Image dir = new Image(getClass().getResourceAsStream(imagePath + "dir.png"));
        List<String> packs;
        if (AppMain.config.getLastdirectory() == null || AppMain.config.getLastdirectory().isEmpty()) {
            packs = new ArrayList<>();
        } else {
            packs = new LoaderPack().getPacks(new File(AppMain.config.getLastdirectory()));
        }
        Node[] nodes = new Node[packs.size()];
        for (int i = 0; i < packs.size(); i++) {
            try {
                final int j = i;
                nodes[i] = FXMLLoader.load(HelloApplication.class.getResource("itempack.fxml"));
                ((Label) nodes[i].lookup("#packname")).setText(packs.get(i).replace(".dnxpack", ""));
                //si c'est un dossier
                ((ImageView) nodes[i].lookup("#packicon")).setImage(packs.get(i).endsWith(".dnxpack") ? dnx : dir);
                vBox.getChildren().add(nodes[i]);
                ((HBox) nodes[j].lookup("#box")).setOnMouseEntered(event -> {
                    ((HBox) nodes[j].lookup("#box")).setStyle("-fx-background-color: #007AFF");
                });
                ((HBox) nodes[j].lookup("#box")).setOnMouseExited(event -> {
                    ((HBox) nodes[j].lookup("#box")).setStyle("-fx-background-color:  #141b35");
                });
                ((ImageView) nodes[i].lookup("#delete")).setOnMouseClicked(event -> {
                    LoaderPack.deletePack(packs.get(j));
                    refresh();
                    event.consume();
                });
                ((ImageView) nodes[i].lookup("#edit")).setOnMouseClicked(event -> {
                    System.out.println("edit " + j);
                    event.consume();
                });
                ((HBox) nodes[j].lookup("#box")).setOnMouseClicked(event -> {
                    packSelected = packs.get(j);
                    LoadPack(packs.get(j));
                });
            } catch (IOException e) {
                e.printStackTrace();
            }
        }
        if (AppMain.config.getLastdirectory() == null || AppMain.config.getLastdirectory().isEmpty()) {
            reload.setVisible(false);
            packadd.setVisible(false);
        }else {
            reload.setVisible(true);
            packadd.setVisible(true);
        }
        if (packSelected == null || packSelected.isEmpty()) {
            add.setVisible(false);
            filter.setVisible(false);
        }else {
            add.setVisible(true);
            filter.setVisible(true);
        }
    }

    @FXML
    public void open() {
        DirectoryChooser fileChooser = new DirectoryChooser();
        fileChooser.setTitle("Open dir pack");
        File selectedFile = fileChooser.showDialog(new Stage());

        if (selectedFile != null && selectedFile.isDirectory()) {
            AppMain.config.setLastdirectory(selectedFile.getAbsolutePath());
            AppMain.config.saveConfig();
            refresh();
        }
    }

    @FXML
    public void refreshB() {
        refresh();
    }

    @FXML
    public void addpack() {
         loadStage("packCreate");
    }

    @FXML
    public void addcontent() {
        loadStage("create");
    }

    @FXML
    public void contact() {
        try {
            Runtime.getRuntime().exec("cmd /c start https://discord.gg/7USXZZTf4h");
        } catch (IOException e) {
            System.out.println("Error while opening discord link");
        }
    }

    @FXML
    public void setting(MouseEvent mouseEvent) {
        loadStage("setting");
    }
}
