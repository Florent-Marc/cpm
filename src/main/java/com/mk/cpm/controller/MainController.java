package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import com.mk.cpm.Main;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.*;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.object.Item;
import com.mk.cpm.loader.object.Vehicul;
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
import java.util.List;
import java.util.ResourceBundle;

public class MainController implements Initializable {

    public static MainController mainController;

    public static Object blockselected;
    public static Stage create;
    public static Stage modify;
    public static Stage createpack;
    public static String packname;
    public Object fileselected;

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
        add.setVisible(false);
        mainController = this;
        remove.setVisible(false);
        remove.setDisable(true);
        choice.setVisible(false);
        choice.getItems().addAll("All","Block", "Armor", "Vehicle", "Item");
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
    public void packSelecter(MouseEvent mouseEvent) {
        if (pack.getSelectionModel().getSelectedItem() == null) {
            return;
        }
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

            for (Object block : Loader.getObject()) {
                if ((block instanceof Block)) {
                    Block b = (Block) block;
                    if (b.getName().equals(list.getSelectionModel().getSelectedItem().substring(3))) {
                        blockselected = block;
                        remove.setDisable(false);
                        ObservableList<String> gf = FXCollections.observableArrayList();
                        for (String s : b.getInfos()) {
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
                if ((block instanceof Armor)) {
                    Armor b = (Armor) block;
                    if (b.getName().equals(list.getSelectionModel().getSelectedItem().substring(3))) {
                        blockselected = b;
                        remove.setDisable(false);
                        ObservableList<String> gf = FXCollections.observableArrayList();
                        for (String s : b.getInfos()) {
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
                if ((block instanceof Vehicul)) {
                    Vehicul b = (Vehicul) block;
                    if (b.getName().equals(list.getSelectionModel().getSelectedItem().substring(3))) {
                        blockselected = b;
                        remove.setDisable(false);
                        ObservableList<String> gf = FXCollections.observableArrayList();
                        for (String s : b.getInfos()) {
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
            if (blockselected instanceof Block) {
                modify = new Stage();
                FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("blockModifier.fxml"));
                Scene scene = null;
                try {
                    scene = new Scene(fxmlLoader.load());
                } catch (IOException e) {
                    throw new RuntimeException(e);
                }
                modify.setTitle("CPM-UI (InProgress)");
                modify.getIcons().add(HelloApplication.logo);
                modify.setScene(scene);
                modify.show();
            }

        }

    }

    @FXML
    public void addObject(MouseEvent mouseEvent) throws IOException {
        Stage stage = new Stage();
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("create.fxml"));
        Scene scene = new Scene(fxmlLoader.load());
        stage.setTitle("CPM-UI");
        stage.getIcons().add(HelloApplication.logo);
        stage.setScene(scene);
        stage.show();
        create = stage;
    }

    //refresh block list
    public void refresh(){
        if (pack.getSelectionModel().getSelectedItem() == null) {
            return;
        }
        int b = 0;
        int a = 0;
        ObservableList<String> l = FXCollections.observableArrayList();

        String choice = this.choice.getSelectionModel().getSelectedItem().toString();
        String packname = pack.getSelectionModel().getSelectedItem().toString();
        if (packname.contains(".dnxpack")) {
            Main.isDynxPack = true;
        }else {
            Main.isDynxPack = false;
        }
        List<Object> objects = Loader.getBlocksByPack(packname);
        if (choice.equals("Block")||choice.equals("All")) {
            for (Object o : objects) {
                if (o instanceof Block) {
                    Block block = (Block) o;
                    l.add("(b)" + block.getName());
                    b++;
                }
            }
        }
        if (choice.equals("Armor")||choice.equals("All")) {
            for (Object o : objects) {
                if (o instanceof Armor) {
                    Armor armor = (Armor) o;
                    l.add("(a)" + armor.getName());
                    a++;
                }
            }
        }
        if (choice.equals("Vehicle")||choice.equals("All")) {
            for (Object o : objects) {
                if (o instanceof Vehicul) {
                    Vehicul vehicul = (Vehicul) o;
                    l.add("(v)" + vehicul.getName());
                }
            }
        }
        if (choice.equals("Item")||choice.equals("All")) {
            for (Object o : objects) {
                if (o instanceof Item) {
                    Item item = (Item) o;
                    //check if list contains item
                    if (!l.contains("(v)" + item.getName())&&!l.contains("(a)" + item.getName())&&!l.contains("(b)" + item.getName())){
                        l.add("(i)" + item.getName());
                    }
                }
            }
        }
        list.getItems().clear();
        list.setItems(l);
        info.getItems().clear();
        left.setText("Contents : " + b + " Blocks and " + a + " Armors");
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
        for (Object block : Loader.getObject()) {
            if (block instanceof Block) {
                Block b = (Block) block;
                if (b.getName().equals(list.getSelectionModel().getSelectedItem())) {
                    //remove dir of block
                    File f = new File("pack/" + packname + "/assets/dynamxmod/models/" + b.getModel()).getParentFile();
                    //remove dir
                    if (f.exists()) {
                        f.deleteOnExit();
                    }
                    //remove .dynx file
                    File f2 = new File("pack/" + packname);
                    //find file in dir
                    for (File f3 : f2.listFiles()) {
                        //pack check pack_info.dynx and assets
                        if (f3.getName().equals("pack_info.dynx") || f3.getName().equals("assets")) {
                            continue;
                        }
                        if (f3.isDirectory()) {
                            for (File f4 : f3.listFiles()) {
                                if (f4.isDirectory()) {
                                    for (File f5 : f4.listFiles()) {
                                        if (f5.getName().equals("block_" + b.getName() + ".dynx")) {
                                            f5.deleteOnExit();
                                            f5.delete();
                                            break;
                                        }
                                    }
                                }
                                if (f4.getName().equals("block_" + b.getName() + ".dynx")) {
                                    f4.deleteOnExit();
                                    f4.delete();
                                    break;
                                }
                            }
                        }
                        if (f3.getName().equals("block_" + b.getName() + ".dynx")) {
                            f3.deleteOnExit();
                            f3.delete();
                            break;
                        }

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
        stage.getIcons().add(HelloApplication.logo);
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
    public void packCreateButton(ActionEvent actionEvent) {

        createpack = new Stage();
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("packCreate.fxml"));
        Scene scene = null;
        try {
            scene = new Scene(fxmlLoader.load());
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        createpack.setTitle("CPM-UI (InProgress)");
        createpack.getIcons().add(HelloApplication.logo);
        createpack.setScene(scene);
        createpack.setResizable(false);
        createpack.show();
    }

    @FXML
    public void choice(ActionEvent actionEvent) {
        refresh();
    }
}
