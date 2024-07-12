package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.control.ScrollPane;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;

import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

public class Test implements Initializable {

    @FXML
    public VBox PackList, vbox2,vbox3;
    public ScrollPane scrollPane, scrollpane2, scrollpane3;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        String imagePath = "/com/mk/cpm/texture/";
        VBox vBox = PackList;
        vBox.setSpacing(5);

// Création du ScrollPane et configuration
        scrollPane.setContent(vBox);
        scrollPane.setFitToWidth(true); // Ajuste le contenu en largeur
        scrollPane.setFitToHeight(true); // Ajuste le contenu en hauteur

// Lié la taille minimale du ScrollPane à celle de la VBox
        scrollPane.minWidthProperty().bind(vBox.minWidthProperty());
        scrollPane.minHeightProperty().bind(vBox.minHeightProperty());
        //retire les barres de scroll
        scrollPane.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollPane.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        //augmenter la viteese de scroll
        scrollPane.setHvalue(1);


        Image dnx = new Image(getClass().getResourceAsStream(imagePath+"dnx.png"));
        Image dir = new Image(getClass().getResourceAsStream(imagePath+"dir.png"));
        Node[] nodes = new Node[25];
        for (int i = 0; i < 25; i++) {
            try {
                final int j = i;
                nodes[i] = FXMLLoader.load(HelloApplication.class.getResource("itempack.fxml"));
                ((Label) nodes[i].lookup("#packname")).setText("Pack " + i);
                ((ImageView) nodes[i].lookup("#packicon")).setImage(i % 2 == 0 ? dnx : dir);
                vBox.getChildren().add(nodes[i]);
                ((HBox) nodes[j].lookup("#box")).setOnMouseEntered(event -> {
                    ((HBox) nodes[j].lookup("#box")).setStyle("-fx-background-color: #007AFF");
                });
                ((HBox) nodes[j].lookup("#box")).setOnMouseExited(event -> {
                    ((HBox) nodes[j].lookup("#box")).setStyle("-fx-background-color:  #141b35");
                });
                ((ImageView) nodes[i].lookup("#edit")).setOnMouseClicked(event -> {
                    System.out.println("edit " + j);
                });
                ((ImageView) nodes[i].lookup("#delete")).setOnMouseClicked(event -> {
                    System.out.println("delete " + j);
                });
                ((HBox) nodes[j].lookup("#box")).setOnMouseClicked(event -> {
                    System.out.println("Pack " + j + " clicked");
                });
            } catch (IOException e) {
                e.printStackTrace();
            }
        }


        //========================================================================================================

        VBox vBox2 = vbox2;
        vBox2.setSpacing(5);

        scrollpane2.setContent(vBox2);
        scrollpane2.setFitToWidth(true);
        scrollpane2.setFitToHeight(true);
        scrollpane2.minWidthProperty().bind(vBox2.minWidthProperty());
        scrollpane2.minHeightProperty().bind(vBox2.minHeightProperty());
        scrollpane2.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollpane2.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        vBox2.prefWidthProperty().bind(scrollpane2.widthProperty());


        Image car = new Image(getClass().getResourceAsStream(imagePath+"car.png"));
        Image block = new Image(getClass().getResourceAsStream(imagePath+"cube.png"));
        Image armor = new Image(getClass().getResourceAsStream(imagePath+"armor.png"));
        Image item = new Image(getClass().getResourceAsStream(imagePath+"axe.png"));
        Node[] nodes2 = new Node[25];
        for (int i = 0; i < 25; i++) {
            try {
                final int j = i;
                nodes2[i] = FXMLLoader.load(HelloApplication.class.getResource("itemcontent.fxml"));
                ((Label) nodes2[i].lookup("#un")).setText("Name : " + i);
                ((Label) nodes2[i].lookup("#deux")).setText("Empty mass : " + i);
                ((Label) nodes2[i].lookup("#trois")).setText("Model : " + i);
                ((ImageView) nodes2[i].lookup("#img")).setImage(i % 4 == 0 ? car : i % 4 == 1 ? block : i % 4 == 2 ? armor : item);
                vBox2.getChildren().add(nodes2[i]);
                ((AnchorPane) nodes2[j].lookup("#box")).setOnMouseEntered(event -> {
                    ((AnchorPane) nodes2[j].lookup("#box")).setStyle("-fx-background-color: #007AFF");
                });
                ((AnchorPane) nodes2[j].lookup("#box")).setOnMouseExited(event -> {
                    ((AnchorPane) nodes2[j].lookup("#box")).setStyle("-fx-background-color:  #141b35");
                });
                ((ImageView) nodes2[i].lookup("#edit")).setOnMouseClicked(event -> {
                    System.out.println("edit " + j);
                });
                ((ImageView) nodes2[i].lookup("#delete")).setOnMouseClicked(event -> {
                    System.out.println("delete " + j);
                });
                ((ImageView) nodes2[i].lookup("#duplicate")).setOnMouseClicked(event -> {
                    System.out.println("duplicate " + j);
                });
                ((AnchorPane) nodes2[j].lookup("#box")).setOnMouseClicked(event -> {
                    System.out.println("Pack " + j + " clicked");
                });
            } catch (IOException e) {
                e.printStackTrace();
            }
        }

        //========================================================================================================


        VBox vBox3 = vbox3;
        vBox3.setSpacing(2);

        scrollpane3.setContent(vBox3);
        scrollpane3.setFitToWidth(true);
        scrollpane3.setFitToHeight(true);
        scrollpane3.minWidthProperty().bind(vBox3.minWidthProperty());
        scrollpane3.minHeightProperty().bind(vBox3.minHeightProperty());
        scrollpane3.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollpane3.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);


        Node[] nodes3 = new Node[30];
        for (int i = 0; i < 30; i++) {
            try {
                nodes3[i] = FXMLLoader.load(HelloApplication.class.getResource("iteminfo.fxml"));
                ((Label) nodes3[i].lookup("#info")).setText("info : " + i);
                vBox3.getChildren().add(nodes3[i]);
            } catch (IOException e) {
                e.printStackTrace();
            }
        }


    }

    @FXML
    public void edit() {

    }
}
