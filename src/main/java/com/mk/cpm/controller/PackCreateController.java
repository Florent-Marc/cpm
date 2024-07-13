package com.mk.cpm.controller;

import com.mk.cpm.config.Config;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ChoiceBox;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.stage.Stage;

import java.io.*;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

public class PackCreateController implements Initializable {


    @FXML
    public CheckBox isdnxpack;
    public TextField name;
    public ChoiceBox list;
    public TextField dcfile;
    public TextField packversion;

    @FXML
    public void create(MouseEvent mouseEvent) {

        File file = null;
        if (isdnxpack.isSelected()){
            file = new File(Config.getLastdirectory()+"\\pack_info.dynx");
            file.getParentFile().mkdirs();
        }else {
            file = new File(Config.getLastdirectory()+"\\"+name.getText()+"\\pack_info.dynx");
            file.getParentFile().mkdirs();
        }
        if (file != null) {
            try {
                FileWriter writer = new FileWriter(file);
                writer.write("PackName: " + name.getText() + "\n");
                writer.write("DcFileVersion: " + list.getSelectionModel().getSelectedItem().toString() + "\n");
                if (dcfile.getText() != null&& !dcfile.getText().isEmpty()){
                    writer.write("CompatibleWithLoaderVersions: " + dcfile.getText() + "\n");
                }
                if (packversion.getText() != null&& !packversion.getText().isEmpty()){
                    writer.write("PackVersion: " + packversion.getText() + "\n");
                }
                writer.flush();
                writer.close();
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        }
        if (isdnxpack.isSelected()){
            // Nom du fichier ZIP à créer
            String zipFileName = name.getText()+".dnxpack";
            try {
                createZipFile(file.getAbsolutePath(), Config.getLastdirectory()+"\\"+zipFileName);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
            file.delete();
        }
        Main.Instance.refresh();
        Stage stage = (Stage) isdnxpack.getScene().getWindow();
        stage.close();
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        list.getItems().addAll("12.5.0");
        list.getSelectionModel().selectFirst();

    }


    public static void createZipFile(String sourceFilePath, String zipFilePath) throws IOException {
        FileOutputStream fos = new FileOutputStream(zipFilePath);
        ZipOutputStream zos = new ZipOutputStream(fos);
        File sourceFile = new File(sourceFilePath);

        try {
            if (sourceFile.isDirectory()) {
                zipDirectory(sourceFile, sourceFile.getName(), zos);
            } else {
                zipFile(sourceFile, zos);
            }
        } finally {
            zos.close();
            fos.close();
        }
    }

    private static void zipDirectory(File folder, String parentFolder, ZipOutputStream zos) throws IOException {
        for (File file : folder.listFiles()) {
            if (file.isDirectory()) {
                zipDirectory(file, parentFolder + "/" + file.getName(), zos);
                continue;
            }
            zos.putNextEntry(new ZipEntry(parentFolder + "/" + file.getName()));
            FileInputStream fis = new FileInputStream(file);
            byte[] buffer = new byte[1024];
            int length;
            while ((length = fis.read(buffer)) >= 0) {
                zos.write(buffer, 0, length);
            }
            zos.closeEntry();
            fis.close();
        }
    }

    private static void zipFile(File file, ZipOutputStream zos) throws IOException {
        zos.putNextEntry(new ZipEntry(file.getName()));
        FileInputStream fis = new FileInputStream(file);
        byte[] buffer = new byte[1024];
        int length;
        while ((length = fis.read(buffer)) >= 0) {
            zos.write(buffer, 0, length);
        }
        zos.closeEntry();
        fis.close();
    }
}
