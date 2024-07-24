package com.mk.cpm.controller;

import com.mk.cpm.AppMain;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.pack.ZipCompressor;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.ChoiceBox;
import javafx.scene.control.Label;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.stage.FileChooser;
import javafx.stage.Stage;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.net.URL;
import java.nio.file.Files;
import java.util.List;
import java.util.ResourceBundle;

public class CreateController implements Initializable {

    private static File fileSelected;
    private static List<File> fileSelected1;
    private static File fileSelected2;

    @FXML
    public Button create;
    public ChoiceBox<String> choicebox;
    public Button select;
    public Label file;
    public Label file1;
    public Label file11;
    public TextField name;
    public TextField desc;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        choicebox.getItems().addAll("Block", "Item", "Armor", "Vehicle");
        //change color of the button red
    }

    @FXML
    public void select(MouseEvent mouseEvent) {
        Stage stage = new Stage();
        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle("Open Resource File");
        File selectedFile = fileChooser.showOpenDialog(stage);

        if (selectedFile != null) {
            // Le fichier a été sélectionné, vous pouvez maintenant l'utiliser
            file.setText(selectedFile.getAbsolutePath());
            fileSelected = selectedFile;
        } else {
            // Aucun fichier n'a été sélectionné
            System.out.println("Aucun fichier sélectionné");
        }
    }

    @FXML
    public void selectpng(MouseEvent mouseEvent) {
        Stage stage = new Stage();
        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle("Open Resource File");
        List<File> selectedFile = fileChooser.showOpenMultipleDialog(stage);

        if (selectedFile == null) {
            System.out.println("Aucun fichier sélectionné");
        } else {
            file1.setText(selectedFile.size() + " File selected");
            fileSelected1 = selectedFile;
        }
    }

    @FXML
    public void selectmtl(MouseEvent mouseEvent) {
        Stage stage = new Stage();
        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle("Open Resource File");
        File selectedFile = fileChooser.showOpenDialog(stage);

        if (selectedFile == null) {
            // Aucun fichier n'a été sélectionné
            System.out.println("Aucun fichier sélectionné");
        } else {
            // Le fichier a été sélectionné, vous pouvez maintenant l'utiliser
            file11.setText(selectedFile.getAbsolutePath());
            fileSelected2 = selectedFile;
        }
    }


    @FXML
    public void se(MouseEvent mouseEvent) {

    }

    @FXML
    public void create(MouseEvent mouseEvent) throws IOException {
        //si tous les choix sont pas remplis mettre le boutton en rouge
        if (fileSelected == null || fileSelected1 == null || fileSelected2 == null || choicebox.getSelectionModel().getSelectedItem() == null) {
            create.setStyle("-fx-background-color: red");
            return;
        }
        //TODO check if file existe deja
        String packname = Main.packSelected;
        String path;
        if (packname.contains(".dnxpack")) {
            path = Config.getCachePath() + "/pack/" + packname;
        } else {
            path = AppMain.config.getLastdirectory() + "/" + packname;
        }
        if (choicebox.getSelectionModel().getSelectedItem().equalsIgnoreCase("Block")) {
            File f = new File(path + "/block/block_" + name.getText() + ".dynx");
            //create the parent directory and the file
            f.getParentFile().mkdirs();
            f.createNewFile();
            Block block = new Block();
            block.setName(name.getText());
            block.setDesc(desc.getText());
            block.setModel("obj/" + name.getText() + "/" + fileSelected.getName());
            block.setFile(f);
            block.save(f);
        }
        if (choicebox.getSelectionModel().getSelectedItem().equalsIgnoreCase("Item")) {
            File f = new File(path + "/item/item_" + name.getText() + ".dynx");
            //create the parent directory and the file
            f.getParentFile().mkdirs();
            f.createNewFile();
            FileWriter writer = new FileWriter(f);
            writer.write("Name: " + name.getText() + "\n");
            writer.write("Desc: " + desc.getText() + "\n");
            writer.write("Model: obj/" + name.getText() + "/" + fileSelected.getName() + "\n");
            writer.write("Texture: obj/" + name.getText() + "/" + fileSelected1.get(0).getName() + "\n");
            writer.write("Texture: obj/" + name.getText() + "/" + fileSelected2.getName() + "\n");
            writer.close();
        }
        if (choicebox.getSelectionModel().getSelectedItem().equalsIgnoreCase("Armor")) {
            File f = new File(path + "/armor/armor_" + name.getText() + ".dynx");
            //create the parent directory and the file
            f.getParentFile().mkdirs();
            f.createNewFile();
            FileWriter writer = new FileWriter(f);
            writer.write("Name: " + name.getText() + "\n");
            writer.write("Desc: " + desc.getText() + "\n");
            writer.write("Model: obj/" + name.getText() + "/" + fileSelected.getName() + "\n");
            writer.write("Texture: obj/" + name.getText() + "/" + fileSelected1.get(0).getName() + "\n");
            writer.write("Texture: obj/" + name.getText() + "/" + fileSelected2.getName() + "\n");
            writer.close();
        }
        if (choicebox.getSelectionModel().getSelectedItem().equalsIgnoreCase("Vehicle")) {
            File f = new File(path + "/vehicle/vehicle_" + name.getText() + ".dynx");
            //create the parent directory and the file
            f.getParentFile().mkdirs();
            f.createNewFile();
            FileWriter writer = new FileWriter(f);
            writer.write("Name: " + name.getText() + "\n");
            writer.write("Desc: " + desc.getText() + "\n");
            writer.write("Model: obj/" + name.getText() + "/" + fileSelected.getName() + "\n");
            writer.write("Texture: obj/" + name.getText() + "/" + fileSelected1.get(0).getName() + "\n");
            writer.write("Texture: obj/" + name.getText() + "/" + fileSelected2.getName() + "\n");
            writer.close();
        }
        //Copy fileselected to the pack
        String target = path + "/assets/dynamxmod/models/obj/" + name.getText() + "/";
        //check if the directory exist
        File f = new File(target);
        f.mkdirs();
        Files.copy(fileSelected.toPath(), new File(target + "/" + fileSelected.getName()).toPath());
        for (File file : fileSelected1) {
            Files.copy(file.toPath(), new File(target +" /" + file.getName()).toPath());
        }
        Files.copy(fileSelected2.toPath(), new File(target + "/" + fileSelected2.getName()).toPath());
        if (packname.contains("\\cpm\\cache")) {
            try {
                String source = Config.getCachePath() + "/pack/" + packname;
                String dest = AppMain.config.getLastdirectory() + "/" + packname;
                ZipCompressor.compressFolder(source, dest);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        }
        Stage stage = (Stage) create.getScene().getWindow();
        stage.close();
        Main.Instance.LoadPack(packname);
    }
}
