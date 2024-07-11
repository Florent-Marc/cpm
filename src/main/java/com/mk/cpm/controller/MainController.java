package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import com.mk.cpm.Main;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.Loader;
import com.mk.cpm.loader.LoaderPack;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.object.Item;
import com.mk.cpm.loader.object.Vehicul;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.input.MouseEvent;
import javafx.stage.DirectoryChooser;
import javafx.stage.Stage;

import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

public class MainController implements Initializable {

    public static MainController Instance;
    public static Object blockselected;
    public static Stage create, modify, createpack;
    public static String packname;

    @FXML
    public ListView<String> list, info;
    public ListView<Object> pack;
    public Label left;
    public Button add, remove;
    public ChoiceBox<String> choice;
    public Menu editmenu;
    public MenuItem newpack;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        Instance = this;
        setupUI();
        loadPacks();
    }

    private void setupUI() {
        editmenu.setDisable(true);
        add.setVisible(false);
        remove.setVisible(false);
        remove.setDisable(true);
        choice.setVisible(false);
        choice.getItems().addAll("All", "Block", "Armor", "Vehicle", "Item");
        choice.getSelectionModel().selectFirst();
        newpack.setDisable(Config.getLastdirectory() == null || Config.getLastdirectory().isEmpty());
    }

    private void loadPacks() {
        if (Config.getLastdirectory() != null && !Config.getLastdirectory().isEmpty()) {
            ObservableList<Object> packs = FXCollections.observableArrayList(new LoaderPack().getPacks(new File(Config.getLastdirectory())));
            pack.setItems(packs);
        }
    }

    @FXML
    public void packSelecter() {
        if (pack.getSelectionModel().getSelectedItem() == null) return;
        remove.setVisible(true);
        remove.setDisable(true);
        choice.setVisible(true);
        refresh();
        packname = pack.getSelectionModel().getSelectedItem().toString();
        add.setVisible(true);
        newpack.setDisable(false);
    }

    @FXML
    public void ObjectSelecter(MouseEvent mouseEvent) {
        if (list.getSelectionModel().getSelectedItem() == null) {
            remove.setDisable(true);
            return;
        }

        blockselected = getObjectByName(list.getSelectionModel().getSelectedItem().substring(3));
        if (blockselected != null) {
            remove.setDisable(false);
            updateInfo(blockselected);
        }

        if (mouseEvent.getClickCount() == 2 && blockselected != null) {
            modify = loadStage(blockselected.getClass().getSimpleName().toLowerCase());
        }
    }

    private Object getObjectByName(String name) {
        for (Object block : Loader.getObject()) {
            if (block instanceof Block && ((Block) block).getName().equals(name)) {
                return block;
            } else if (block instanceof Armor && ((Armor) block).getName().equals(name)) {
                return block;
            } else if (block instanceof Vehicul && ((Vehicul) block).getName().equals(name)) {
                return block;
            }
        }
        return null;
    }

    private void updateInfo(Object block) {
        ObservableList<String> gf = FXCollections.observableArrayList();
        if (block instanceof Block) {
            gf.addAll(((Block) block).getInfos());
        } else if (block instanceof Armor) {
            gf.addAll(((Armor) block).getInfos());
        } else if (block instanceof Vehicul) {
            gf.addAll(((Vehicul) block).getInfos());
        }
        gf.removeIf(s -> s == null || s.contains("null"));
        info.setItems(gf);
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

    @FXML
    public void addObject() {
        create = loadStage("create");
    }

    public void refresh() {
        if (pack.getSelectionModel().getSelectedItem() == null) {
            return;
        }
        ObservableList<String> l = FXCollections.observableArrayList();
        String selectedChoice = choice.getSelectionModel().getSelectedItem();
        String packName = pack.getSelectionModel().getSelectedItem().toString();
        Main.isDynxPack = packName.contains(".dnxpack");

        List<Object> objects = Loader.getBlocksByPack(packName);
        int b = 0, a = 0, v = 0, i = 0;

        for (Object o : objects) {
            if (o instanceof Block && (selectedChoice.equals("Block") || selectedChoice.equals("All"))) {
                l.add("(b)" + ((Block) o).getName());
                b++;
            } else if (o instanceof Armor && (selectedChoice.equals("Armor") || selectedChoice.equals("All"))) {
                l.add("(a)" + ((Armor) o).getName());
                a++;
            } else if (o instanceof Vehicul && (selectedChoice.equals("Vehicle") || selectedChoice.equals("All"))) {
                l.add("(v)" + ((Vehicul) o).getName());
                v++;
            } else if (o instanceof Item && (selectedChoice.equals("Item") || selectedChoice.equals("All"))) {
                if (!l.contains("(v)" + ((Item) o).getName()) && !l.contains("(a)" + ((Item) o).getName()) && !l.contains("(b)" + ((Item) o).getName())) {
                    l.add("(i)" + ((Item) o).getName());
                    i++;
                }
            }
        }
        list.setItems(l);
        info.getItems().clear();
        left.setText(String.format("Contents : %d Blocks %d Armors %d Vehicles %d Items", b, a, v, i));
    }

    @FXML
    public void refreshpack() {
        loadPacks();
        list.getItems().clear();
        info.getItems().clear();
    }

    @FXML
    public void remove() {
        if (list.getSelectionModel().getSelectedItem() == null) {
            return;
        }
        String selectedName = list.getSelectionModel().getSelectedItem().substring(3);
        Object objectToRemove = getObjectByName(selectedName);
        if (objectToRemove != null) {
            Loader.removeObject(objectToRemove, pack.getSelectionModel().getSelectedItem().toString());
            refresh();
        }
    }

    @FXML
    public void open() {
        DirectoryChooser fileChooser = new DirectoryChooser();
        fileChooser.setTitle("Open dir pack");
        File selectedFile = fileChooser.showDialog(new Stage());

        if (selectedFile != null && selectedFile.isDirectory()) {
            ObservableList<Object> packs = FXCollections.observableArrayList(new LoaderPack().getPacks(selectedFile));
            pack.setItems(packs);
            list.getItems().clear();
            Config.setLastdirectory(selectedFile.getAbsolutePath());
            Config.saveConfig();
        }
    }

    @FXML
    public void quit() {
        System.exit(0);
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
    public void packCreateButton() {
        createpack = loadStage("packCreate");
    }

    @FXML
    public void choice() {
        refresh();
    }
}