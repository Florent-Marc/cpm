package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import com.mk.cpm.loader.LoaderAddon;
import com.mk.cpm.loader.MultiParam;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.control.ScrollPane;
import javafx.scene.layout.VBox;

import java.io.IOException;
import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

public class listaddonController implements Initializable {

    @FXML
    public VBox vbox1;
    public ScrollPane scrollpane1;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        vbox1.getChildren().clear();
        vbox1.setSpacing(5);

        scrollpane1.setContent(vbox1);
        scrollpane1.setFitToWidth(true);
        scrollpane1.setFitToHeight(true);
        scrollpane1.minWidthProperty().bind(vbox1.minWidthProperty());
        scrollpane1.minHeightProperty().bind(vbox1.minHeightProperty());
        scrollpane1.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        scrollpane1.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        vbox1.prefWidthProperty().bind(scrollpane1.widthProperty());
        LoaderAddon loaderAddon = new LoaderAddon();
        loaderAddon.load();
        List<Object> objects =  loaderAddon.getVehicles();
        Node[] nodes2 = new Node[objects.size()];
        for (int i = 0; i < objects.size(); i++) {
            //add category
            try {
                nodes2[i] = FXMLLoader.load(HelloApplication.class.getResource("param.fxml"));
                ((Label) nodes2[i].lookup("#paramname")).setText(((MultiParam)objects.get(i)).getName());
                List<String> list = ((MultiParam) objects.get(i)).getList();
                Node[] nodes3 = new Node[list.size()];

                ScrollPane scrollPane = (ScrollPane) nodes2[i].lookup("#scrollpane2");
                VBox vbox2 = (VBox) nodes2[i].lookup("#vbox2");
                if (vbox2 == null) {
                    vbox2 = new VBox();
                }
                vbox2.getChildren().clear();
                vbox2.setSpacing(5);
                scrollPane.setContent(vbox2);
                scrollPane.setFitToWidth(true);
                scrollPane.setFitToHeight(true);
                scrollPane.minWidthProperty().bind(vbox2.minWidthProperty());
                scrollPane.minHeightProperty().bind(vbox2.minHeightProperty());
                scrollPane.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
                scrollPane.setVbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
                vbox2.prefWidthProperty().bind(scrollPane.widthProperty());
//
//                for (int j = 0; j < list.size(); j++) {
//                    try {
//                        nodes3[j] = FXMLLoader.load(HelloApplication.class.getResource("itemparam.fxml"));
//                        ((Label) nodes3[j].lookup("#name")).setText(list.get(j));
//                        vbox2.getChildren().add(nodes3[j]);
//                    } catch (IOException e) {
//                        throw new RuntimeException(e);
//                    }
//                    vbox2.getChildren().add(nodes3[j]);
//                }
            } catch (IOException e) {
                throw new RuntimeException(e);
            }


            vbox1.getChildren().add(nodes2[i]);

        }
    }
}
