package com.mk.cpm.controller;

import com.mk.cpm.config.Config;
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
import java.util.ResourceBundle;

public class CreateController implements Initializable{

    private static File fileSelected;
    private static File fileSelected1;
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
        choicebox.getItems().addAll("Block");
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
            System.out.println("Fichier sélectionné: " + selectedFile.getAbsolutePath());
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
        File selectedFile = fileChooser.showOpenDialog(stage);

        if (selectedFile != null) {
            // Le fichier a été sélectionné, vous pouvez maintenant l'utiliser
            file1.setText(selectedFile.getAbsolutePath());
            fileSelected1 = selectedFile;
        } else {
            // Aucun fichier n'a été sélectionné
            System.out.println("Aucun fichier sélectionné");
        }
    }

    @FXML
    public void selectmtl(MouseEvent mouseEvent) {
        Stage stage = new Stage();
        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle("Open Resource File");
        File selectedFile = fileChooser.showOpenDialog(stage);

        if (selectedFile != null) {
            // Le fichier a été sélectionné, vous pouvez maintenant l'utiliser
            file11.setText(selectedFile.getAbsolutePath());
            fileSelected2 = selectedFile;
        } else {
            // Aucun fichier n'a été sélectionné
            System.out.println("Aucun fichier sélectionné");
        }
    }


    @FXML
    public void se(MouseEvent mouseEvent) {

    }

    @FXML
    public void create(MouseEvent mouseEvent) throws IOException {
        //si tous les choix sont pas remplis mettre le boutton en rouge
        if (fileSelected == null || fileSelected1 == null || fileSelected2 == null || choicebox.getSelectionModel().getSelectedItem() == null){
            create.setStyle("-fx-background-color: red");
            return;
        }
        //TODO check if file existe deja
        Object packname = MainController.packname;
        //check if dir blocks exist
        //if not create it
        if (!new File(Config.getLastdirectory()+"/"+packname+"/blocks").exists()){
            new File(Config.getLastdirectory()+"/"+packname+"/blocks").mkdirs();
        }
        //check if dir assets/dynamxmod/models/blocks exist
        if (!new File(Config.getLastdirectory()+"/"+packname+"/assets/dynamxmod/models/blocks/"+name.getText()).exists()){
            new File(Config.getLastdirectory()+"/"+packname+"/assets/dynamxmod/models/blocks/"+name.getText()).mkdirs();
        }
        //copy file in the directory
        //copy fileSelected in assets/dynamxmod/models/blocks/name
        Files.copy(fileSelected.toPath(), new File(Config.getLastdirectory()+"/"+packname+"/assets/dynamxmod/models/blocks/"+name.getText()+"/"+fileSelected.getName()).toPath());
        //copy fileSelected1 in assets/dynamxmod/models/blocks/name
        Files.copy(fileSelected1.toPath(), new File(Config.getLastdirectory()+"/"+packname+"/assets/dynamxmod/models/blocks/"+name.getText()+"/"+fileSelected1.getName()).toPath());
        //copy fileSelected2 in assets/dynamxmod/models/blocks/name
        Files.copy(fileSelected2.toPath(), new File(Config.getLastdirectory()+"/"+packname+"/assets/dynamxmod/models/blocks/"+name.getText()+"/"+fileSelected2.getName()).toPath());

        //create file block_name.dynx
        File f = new File(Config.getLastdirectory()+"/"+packname+"/blocks/block_"+name.getText()+".dynx");
        //write in the file
        FileWriter fw = new FileWriter(f);
        fw.write("Name: "+name.getText()+"\n");
        fw.write("Description: "+desc.getText()+"\n");
        fw.write("Model: blocks/"+name.getText()+"/"+fileSelected.getName()+"\n");
        fw.close();
        MainController.create.close();
        MainController.mainController.refresh();
    }
}
