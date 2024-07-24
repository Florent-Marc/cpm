package com.mk.cpm.controller;

import com.interactivemesh.jfx.importer.obj.ObjModelImporter;
import com.mk.cpm.AppMain;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.Loader;
import com.mk.cpm.loader.object.Armor;
import com.mk.cpm.loader.pack.ZipCompressor;
import javafx.animation.KeyFrame;
import javafx.animation.KeyValue;
import javafx.animation.Timeline;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.*;
import javafx.scene.control.ChoiceBox;
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

public class ArmorController implements Initializable {

    public Pane render;
    private Armor armor;
    @FXML
    public ChoiceBox<Object> choiceBox;
    public TextField name;
    public TextField desc;
    public TextField model;
    public TextField CreativeTab;
    public TextField itemrotation;
    public TextField itemtranslation;
    public TextField itemscale;
    public TextField IconText;
    public TextField ArmorArms;
    public TextField ArmorBody;
    public TextField ArmorFoot;
    public TextField ArmorHead;
    public TextField ArmorLegs;
    public TextField DamageReduction;
    public TextField Durability;
    public TextField Textures;
    public TextField Toughness;
    public TextField Enchantability;
    public TextField EquipSound;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        if (Main.objectSelect == null) {
            return;
        }
        armor = (Armor) Main.objectSelect;
        if (armor == null) {
            return;
        }
        if (armor.getCreativeTab() != null) {
            CreativeTab.setText(armor.getCreativeTab());
        }
        if (armor.getModel() != null) {
            model.setText(armor.getModel());
        }
        if (armor.getDesc() != null) {
            desc.setText(armor.getDesc());
        }
        if (armor.getName() != null) {
            name.setText(armor.getName());
        }
        if (armor.getItemRotation() != null) {
            itemrotation.setText(armor.getItemRotation());
        }
        if (armor.getItem().getItemTranslate() != null) {
            itemtranslation.setText(armor.getItem().getItemTranslate());
        }
        if (armor.getItemScale() != null) {
            itemscale.setText(armor.getItemScale());
        }
        choiceBox.getItems().addAll("NONE", "WORLD", "ALL");
        choiceBox.getSelectionModel().selectFirst();
        if (armor.getIconText() != null) {
            IconText.setText(armor.getIconText());
        }
        if (armor.getArmorArms() != null) {
            ArmorArms.setText(armor.getArmorArms());
        }
        if (armor.getArmorBody() != null) {
            ArmorBody.setText(armor.getArmorBody());
        }
        if (armor.getArmorFoot() != null) {
            ArmorFoot.setText(armor.getArmorFoot());
        }
        if (armor.getArmorHead() != null) {
            ArmorHead.setText(armor.getArmorHead());
        }
        if (armor.getArmorLegs() != null) {
            ArmorLegs.setText(armor.getArmorLegs());
        }
        if (armor.getDamageReduction() != null) {
            DamageReduction.setText(armor.getDamageReduction());
        }
        if (armor.getDurability() != null) {
            Durability.setText(armor.getDurability());
        }
        if (armor.getTextures() != null) {
            Textures.setText(armor.getTextures());
        }
        if (armor.getToughness() != null) {
            Toughness.setText(armor.getToughness());
        }
        if (armor.getEnchantability() != null) {
            Enchantability.setText(armor.getEnchantability());
        }
        if (armor.getEquipSound() != null) {
            EquipSound.setText(armor.getEquipSound());
        }
        ObjModelImporter myModel = new ObjModelImporter();
        try {
            //get only the name of the model /cc/test/test.obj -> test
            String[] split = armor.getModel().split("/");
            File file = new File(Loader.getPath(split[split.length - 1], Main.packSelected));
            System.out.println("Loading model from: " + file.getAbsolutePath());
            myModel.read(file);
        } catch (Exception e) {
            e.printStackTrace();
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
    public void save(MouseEvent mouseEvent) {
        if (armor == null) {
            return;
        }
        armor.setCreativeTab(CreativeTab.getText());
        armor.setModel(model.getText());
        armor.setDesc(desc.getText());
        armor.setName(name.getText());
        armor.setItemRotation(itemrotation.getText());
        armor.getItem().setItemTranslate(itemtranslation.getText());
        armor.setItemScale(itemscale.getText());
        armor.setIconText(IconText.getText());
        armor.setArmorArms(ArmorArms.getText());
        armor.setArmorBody(ArmorBody.getText());
        armor.setArmorFoot(ArmorFoot.getText());
        armor.setArmorHead(ArmorHead.getText());
        armor.setArmorLegs(ArmorLegs.getText());
        armor.setDamageReduction(DamageReduction.getText());
        armor.setDurability(Durability.getText());
        armor.setTextures(Textures.getText());
        armor.setToughness(Toughness.getText());
        armor.setEnchantability(Enchantability.getText());
        armor.setEquipSound(EquipSound.getText());
        File f = armor.getFile();
        if (f == null) {
            return;
        }

        try {
            FileWriter fileWriter = new FileWriter(f);
            fileWriter.write("");
            fileWriter.close();
        } catch (Exception e) {
            e.printStackTrace();
        }
        this.armor.save(f);
        if (f.getPath().contains("\\cpm\\cache")) {
            try {
                String source = Config.getCachePath() + "/pack/" + Main.packSelected;
                String dest = AppMain.config.getLastdirectory() + "/" + Main.packSelected;
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
