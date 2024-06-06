package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.*;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.object.Block;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
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
import java.util.ResourceBundle;

public class MainController implements Initializable {

    public static MainController mainController;

    public static Object blockselected;
    public static Stage create;
    public static Stage modify;
    public static Stage createpack;
    public static Object packname;

    @FXML
    public ListView<String> list;
    public ListView<Object> pack;
    public ListView<String> info;
    public Label left;
    public Button add;
    public Button remove;
    public ChoiceBox choice;
    public Menu editmenu;
    public MenuItem newpack;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        ObservableList<String> l = FXCollections.observableArrayList();
        editmenu.setDisable(true);
        //l.addAll(new LoaderPack().getPacks());
        //pack.setItems(l);
        add.setVisible(false);
        mainController = this;
        remove.setVisible(false);
        remove.setDisable(true);
        choice.setVisible(false);
        choice.getItems().addAll("All","Block", "Armor");
        choice.getSelectionModel().selectFirst();
        if (Config.getLastdirectory() != null) {
            if (Config.getLastdirectory().isEmpty()) {
                return;
            }
            ObservableList<Object> l2 = FXCollections.observableArrayList();
            l2.addAll(new LoaderPack().getPacks(new File(Config.getLastdirectory())));
            pack.setItems(l2);
            newpack.setDisable(false);
        }else {
            newpack.setDisable(true);
        }
    }


    @FXML
    public void pack(MouseEvent mouseEvent) {
        if (pack.getSelectionModel().getSelectedItem() == null) {
            return;
        }
        remove.setVisible(true);
        remove.setDisable(true);
        choice.setVisible(true);
        int b = 0;
        int a = 0;
        ObservableList<String> l = FXCollections.observableArrayList();
        String choice = this.choice.getSelectionModel().getSelectedItem().toString();
        if (choice.equals("Block")||choice.equals("All")) {
            for (Block block : LoaderBlock.getBlocksByPack(pack.getSelectionModel().getSelectedItem())) {
                l.add("(b)" + block.getName());
                b++;
            }
        }
        if (choice.equals("Armor")||choice.equals("All")) {
            for (Armor armor : LoaderArmor.getArmorByPack(pack.getSelectionModel().getSelectedItem())) {
                l.add("(a)" + armor.getName());
                a++;
            }
        }
        list.setItems(l);
        info.getItems().clear();
        left.setText("Contents : " + b + " Blocks and " + a + " Armors");
        packname = pack.getSelectionModel().getSelectedItem();
        add.setVisible(true);
        newpack.setDisable(false);
    }

    @FXML
    public void listf(MouseEvent mouseEvent) {
        if (list.getSelectionModel().getSelectedItem() == null) {
            remove.setDisable(true);
            return;
        }
        if (list.getSelectionModel().getSelectedItem().contains("(b)")) {
            for (Block block : LoaderBlock.getBlocks()) {
                if (block.getName().equals(list.getSelectionModel().getSelectedItem().substring(3))) {
                    blockselected = block;
                    remove.setDisable(false);
                    ObservableList<String> gf = FXCollections.observableArrayList();
                    for (String s : block.getInfos()) {
                        //check if null
                        if (s == null) {
                            continue;
                        }
                        if (s.contains("null")) {
                            continue;
                        }
                        gf.add(s);
                    }
                    info.setItems(gf);

                }
            }
        }
        if (list.getSelectionModel().getSelectedItem().contains("(a)")) {
            for (Armor armor : LoaderArmor.getArmor()) {
                if (armor.getName().equals(list.getSelectionModel().getSelectedItem().substring(3))) {
                    blockselected = armor;
                    remove.setDisable(false);
                    ObservableList<String> gf = FXCollections.observableArrayList();
                    for (String s : armor.getInfos()) {
                        //check if null
                        if (s == null) {
                            continue;
                        }
                        if (s.contains("null")) {
                            continue;
                        }
                        gf.add(s);
                    }
                    info.setItems(gf);

                }
            }
        }

        if (mouseEvent.getClickCount() == 2) {

            modify = new Stage();
            FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("blockModifier.fxml"));
            Scene scene = null;
            try {
                scene = new Scene(fxmlLoader.load());
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
            modify.setTitle("CPM-UI (InProgress)");
            modify.setScene(scene);
            modify.show();

        }

    }

    @FXML
    public void add(MouseEvent mouseEvent) throws IOException {
        Stage stage = new Stage();
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("create.fxml"));
        Scene scene = new Scene(fxmlLoader.load());
        stage.setTitle("CPM-UI");
        stage.setScene(scene);
        stage.show();
        create = stage;
    }

    //refresh block list
    public void refresh(){
        ObservableList<String> l = FXCollections.observableArrayList();
        String choice = this.choice.getSelectionModel().getSelectedItem().toString();
        if (choice.equals("Block")||choice.equals("All")) {
            for (Block block : LoaderBlock.getBlocksByPack(packname)) {
                l.add("(b)" + block.getName());
            }
        }
        if (choice.equals("Armor")||choice.equals("All")) {
            for (Armor armor : LoaderArmor.getArmorByPack(packname)) {
                l.add("(a)" + armor.getName());
            }
        }
        this.list.setItems(l);
    }
    //refresh pack list
    public void refreshpack(){
        ObservableList<Object> l = FXCollections.observableArrayList();
        l.addAll(new LoaderPack().getPacks(new File(Config.getLastdirectory())));
        this.pack.setItems(l);
    }


    @FXML
    public void remove(MouseEvent mouseEvent) {
        if (list.getSelectionModel().getSelectedItem() == null) {
            return;
        }
        for (Block block : LoaderBlock.getBlocks()) {
            if (block.getName().equals(list.getSelectionModel().getSelectedItem())) {
                //remove dir of block
                File f = new File("pack/" + packname + "/assets/dynamxmod/models/" + block.getModel()).getParentFile();
                //remove dir
                if (f.exists()) {
                    f.deleteOnExit();
                }
                //remove .dynx file
                File f2 = new File("pack/" + packname);
                //find file in dir
                for (File f3 : f2.listFiles()) {
                    //pack check pack_info.dynx and assets
                    if(f3.getName().equals("pack_info.dynx")||f3.getName().equals("assets")){
                        continue;
                    }
                    if (f3.isDirectory()){
                        for (File f4 : f3.listFiles()){
                            if(f4.isDirectory()){
                                for (File f5 : f4.listFiles()) {
                                    if (f5.getName().equals("block_"+block.getName() + ".dynx")) {
                                        f5.deleteOnExit();
                                        f5.delete();
                                        break;
                                    }
                                }
                            }
                            if (f4.getName().equals("block_"+block.getName() + ".dynx")) {
                                f4.deleteOnExit();
                                f4.delete();
                                break;
                            }
                        }
                    }
                    if (f3.getName().equals("block_"+block.getName() + ".dynx")) {
                        f3.deleteOnExit();
                        f3.delete();
                        break;
                    }

                }
            }
        }
        refresh();
    }


    @FXML
    public void open(ActionEvent actionEvent) {
        System.out.println("open");
        Stage stage = new Stage();
        DirectoryChooser fileChooser = new DirectoryChooser();
        fileChooser.setTitle("Open dir pack");
        File selectedFile = fileChooser.showDialog(stage);

        if (selectedFile == null) {
            return;
        }
        if(selectedFile.isDirectory()){
            ObservableList<Object> l = FXCollections.observableArrayList();
            l.addAll(new LoaderPack().getPacks(selectedFile));
            pack.setItems(l);
            list.getItems().clear();
            Config.setLastdirectory(selectedFile.getAbsolutePath());
            Config.saveConfig();
        }
    }

    @FXML
    public void quit(ActionEvent actionEvent) {
        System.exit(0);
    }

    @FXML
    public void contact(ActionEvent actionEvent) {
        //open discord
        try {
            Runtime.getRuntime().exec("cmd /c start https://discord.gg/7USXZZTf4h");
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    @FXML
    public void ne(ActionEvent actionEvent) {

        createpack = new Stage();
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("packCreate.fxml"));
        Scene scene = null;
        try {
            scene = new Scene(fxmlLoader.load());
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        createpack.setTitle("CPM-UI (InProgress)");
        createpack.setScene(scene);
        createpack.setResizable(false);
        createpack.show();
    }

    @FXML
    public void choice(ActionEvent actionEvent) {
        refresh();
    }
}
