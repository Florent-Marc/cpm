package com.mk.cpm.controller;

import com.interactivemesh.jfx.importer.obj.ObjModelImporter;
import com.mk.cpm.AppMain;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.Loader;
import com.mk.cpm.loader.object.Block;
import com.mk.cpm.loader.pack.ZipCompressor;
import javafx.animation.KeyFrame;
import javafx.animation.KeyValue;
import javafx.animation.Timeline;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.*;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ChoiceBox;
import javafx.scene.control.Tab;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import javafx.scene.paint.PhongMaterial;
import javafx.scene.shape.MeshView;
import javafx.scene.transform.Rotate;
import javafx.stage.Stage;
import javafx.util.Duration;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

public class BlockController implements Initializable {

    public Pane render;
    private Block block;
    @FXML
    public Tab tabprop;
    public CheckBox isprop;
    public TextField name;
    public TextField desc;
    public TextField model;
    public TextField CreativeTab;
    public ChoiceBox choiceBox;
    public TextField itemrotation;
    public TextField itemtranslation;
    public TextField itemscale;
    public TextField IconText;
    public TextField LightLevel;
    public TextField Material;
    public TextField RenderDistance;
    public TextField Scale;
    public TextField Rotation;
    public TextField Translation;
    public TextField Textures;
    public TextField UseComplexCollision;
    public TextField EmptyMass;
    public TextField CenterOfGravityOffset;
    public TextField RespawnTime;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        if (Main.objectSelect == null) {
            return;
        }
        block = (Block) Main.objectSelect;
        if (block == null) {
            Stage stage = (Stage) name.getScene().getWindow();
            stage.close();
        }
        if (block.getEmptyMass() != null) {
            tabprop.setDisable(false);
            isprop.setSelected(true);
        } else {
            tabprop.setDisable(true);
        }
        if (block.getCreativeTab() != null) {
            CreativeTab.setText(block.getCreativeTab());
        }
        if (block.getModel() != null) {
            model.setText(block.getModel());
        }
        if (block.getDesc() != null) {
            desc.setText(block.getDesc());
        }
        if (block.getName() != null) {
            name.setText(block.getName());
        }
        if (block.getItemRotation() != null) {
            itemrotation.setText(block.getItemRotation());
        }
        if (block.getItem().getItemTranslate() != null) {
            itemtranslation.setText(block.getItem().getItemTranslate());
        }
        if (block.getItemScale() != null) {
            itemscale.setText(block.getItemScale());
        }
        choiceBox.getItems().addAll("NONE", "WORLD", "ALL");
        choiceBox.getSelectionModel().selectFirst();
        if (block.getLightLevel() != null) {
            LightLevel.setText(block.getLightLevel());
        }
        if (block.getMaterial() != null) {
            Material.setText(block.getMaterial());
        }
        if (block.getRenderDistance() != null) {
            RenderDistance.setText(block.getRenderDistance());
        }
        if (block.getScale() != null) {
            Scale.setText(block.getScale());
        }
        if (block.getRotation() != null) {
            Rotation.setText(block.getRotation());
        }
        if (block.getTranslation() != null) {
            Translation.setText(block.getTranslation());
        }
        if (block.getTextures() != null) {
            Textures.setText(block.getTextures());
        }
        if (block.getUseComplexCollision() != null) {
            UseComplexCollision.setText(String.valueOf(block.getUseComplexCollision()));
        }
        if (block.getEmptyMass() != null) {
            EmptyMass.setText(block.getEmptyMass());
        }
        if (block.getCenterOfGravityOffset() != null) {
            CenterOfGravityOffset.setText(block.getCenterOfGravityOffset());
        }
        if (block.getDespawnTime() != null) {
            RespawnTime.setText(block.getDespawnTime());
        }
        //icon
        if (block.getIconText() != null) {
            IconText.setText(block.getIconText());
        }
        ObjModelImporter myModel = new ObjModelImporter();
        try {
            //get only the name of the model /cc/test/test.obj -> test
            String[] split = block.getModel().split("/");
            File file = new File(Loader.getPath(split[split.length - 1], Main.packSelected));
            System.out.println("Loading model from: " + file.getAbsolutePath());
            myModel.read(file);
        } catch (Exception e) {
            System.out.println("Error loading model");
            //e.printStackTrace();
            return;
        }

        MeshView[] meshViews = myModel.getImport();
        if (meshViews == null || meshViews.length == 0) {
            System.err.println("No meshes found in the model.");
            return;
        }

        Group root = new Group(meshViews);


        for (MeshView meshView : meshViews) {
            PhongMaterial material = new PhongMaterial();
            material.setDiffuseColor(Color.color(Math.random(), Math.random(), Math.random()));
            meshView.setMaterial(material);
        }

        Camera camera = new PerspectiveCamera(true);
        camera.setTranslateZ(-10);

        PointLight light = new PointLight(Color.WHITE);
        light.setTranslateX(-180);
        light.setTranslateY(-90);
        light.setTranslateZ(-120);
        root.getChildren().add(light);

        // Create a light source a l'oppose du point de vue de la camera
        PointLight light2 = new PointLight(Color.WHITE);
        light2.setTranslateX(180);
        light2.setTranslateY(90);
        light2.setTranslateZ(120);
        root.getChildren().add(light2);

        SubScene subScene = new SubScene(root, 800, 600, true, SceneAntialiasing.BALANCED);
        subScene.setFill(Color.DARKGRAY);
        subScene.setCamera(camera);
        subScene.widthProperty().bind(render.widthProperty());
        subScene.heightProperty().bind(render.heightProperty());
        render.getChildren().add(subScene);


        Rotate rot = new Rotate(0, Rotate.Y_AXIS);
        root.getTransforms().add(rot);

        Timeline anim = new Timeline(
                new KeyFrame(Duration.seconds(0), new KeyValue(rot.angleProperty(), 0)),
                new KeyFrame(Duration.seconds(15), new KeyValue(rot.angleProperty(), 360))
        );
        anim.setCycleCount(Timeline.INDEFINITE);
        anim.play();
    }

    @FXML
    public void props(MouseEvent mouseEvent) {
        if (isprop.isSelected()) {
            tabprop.setDisable(false);
        } else {
            tabprop.setDisable(true);
        }
    }

    @FXML
    public void save(MouseEvent mouseEvent) {
        if (block == null) {
            return;
        }
        block.setCreativeTab(CreativeTab.getText());
        block.setModel(model.getText());
        block.setDesc(desc.getText());
        block.setName(name.getText());
        block.setItemRotation(itemrotation.getText());
        block.getItem().setItemTranslate(itemtranslation.getText());
        block.setItemScale(itemscale.getText());
        block.setLightLevel(LightLevel.getText());
        block.setMaterial(Material.getText());
        block.setRenderDistance(RenderDistance.getText());
        block.setScale(Scale.getText());
        block.setRotation(Rotation.getText());
        block.setTranslation(Translation.getText());
        block.setTextures(Textures.getText());
        block.setUseComplexCollision(Boolean.parseBoolean(UseComplexCollision.getText()));
        if (isprop.isSelected()) {
            block.setEmptyMass(EmptyMass.getText());
            block.setCenterOfGravityOffset(CenterOfGravityOffset.getText());
            block.setDespawnTime(RespawnTime.getText());
        } else {
            block.setEmptyMass(null);
            block.setCenterOfGravityOffset(null);
            block.setDespawnTime(null);
        }
        block.setIconText(IconText.getText());
        block.setCreativeTab(CreativeTab.getText());
        File f = block.getFile();
        if (f == null) {
            return;
        }
        //clear file
        try {
            FileWriter fileWriter = new FileWriter(f);
            fileWriter.write("");
            fileWriter.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
        this.block.save(f);
        if (f.getPath().contains("\\cpm\\cache")) {
            try {
                String source = Config.getCachePath() + "/pack/" + Main.packSelected;
                ;
                String dest = AppMain.config.getLastdirectory() + "/" + Main.packSelected;
                ;
                ZipCompressor.compressFolder(source, dest);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        }
        Stage stage = (Stage) name.getScene().getWindow();
        stage.close();
        Main t = Main.Instance;
        t.LoadPack(Main.packSelected);

    }
}