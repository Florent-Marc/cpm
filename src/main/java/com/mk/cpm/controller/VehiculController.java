package com.mk.cpm.controller;

import com.mk.cpm.loader.object.Vehicul;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ChoiceBox;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;

import java.net.URL;
import java.util.ResourceBundle;

public class VehiculController implements Initializable {

    private Vehicul vehicul;

    @FXML
    //item
    public TextField CreativeTab;
    public ChoiceBox choiceBox;
    public TextField itemrotation;
    public TextField itemtranslation;
    public TextField itemscale;
    public TextField IconText;
    //base
    public TextField name;
    public TextField desc;
    public TextField model;
    public TextField DragCoefficient;
    public TextField EmptyMass;
    public TextField AngularDamping;
    public TextField LinearDamping;
    public TextField DefaultEngine;
    public TextField DefaultSounds;
    public TextField DefaultZoomLevel;
    public TextField MaxVehicleSpeed;
    public TextField PlayerStandOnTop;
    public TextField ScaleModifier;
    public TextField ShapeYOffset;
    public TextField CenterOfGravityOffset;
    //wheels
    public TextField Namew;
    public TextField AttachedWheel;
    public TextField DrivingWheel;
    public TextField IsSteerable;
    public TextField MaxTurn;
    public TextField Position;
    public TextField DependsOn;
    public TextField MudGuard;
    public TextField RotationPoint;
    public TextField Scale;
    public TextField Suspension;
    //shape
    public TextField Names;
    public TextField Scales;
    public TextField Positions;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        if (MainController.blockselected == null) {
            return;
        }
        vehicul = (Vehicul) MainController.blockselected;
        if (vehicul == null) {
            return;
        }
        if (vehicul.getName()!=null) {
            name.setText(vehicul.getName());
        }
        if (vehicul.getDesc()!=null) {
            desc.setText(vehicul.getDesc());
        }
        if (vehicul.getModel()!=null) {
            model.setText(vehicul.getModel());
        }

        if (vehicul.getCreativeTab() != null) {
            CreativeTab.setText(vehicul.getCreativeTab());
        }
        if (vehicul.getIconText() != null) {
            IconText.setText(vehicul.getIconText());
        }
        if (vehicul.getItemRotation() != null) {
            itemrotation.setText(vehicul.getItemRotation());
        }
        if (vehicul.getItem().getItemTranslate() != null) {
            itemtranslation.setText(vehicul.getItem().getItemTranslate());
        }
        if (vehicul.getItemScale() != null) {
            itemscale.setText(vehicul.getItemScale());
        }
        choiceBox.getItems().addAll("NONE", "WORLD", "ALL");
        choiceBox.getSelectionModel().selectFirst();
        if (vehicul.getDragCoefficient() != null) {
            DragCoefficient.setText(vehicul.getDragCoefficient());
        }
        if (vehicul.getEmptyMass() != null) {
            EmptyMass.setText(vehicul.getEmptyMass());
        }
        if (vehicul.getAngularDamping() != null) {
            AngularDamping.setText(vehicul.getAngularDamping());
        }
        if (vehicul.getLinearDamping() != null) {
            LinearDamping.setText(vehicul.getLinearDamping());
        }
        if (vehicul.getDefaultEngine() != null) {
            DefaultEngine.setText(vehicul.getDefaultEngine());
        }
        if (vehicul.getDefaultSounds() != null) {
            DefaultSounds.setText(vehicul.getDefaultSounds());
        }
        if (vehicul.getDefaultZoomLevel() != null) {
            DefaultZoomLevel.setText(vehicul.getDefaultZoomLevel());
        }
        if (vehicul.getMaxVehicleSpeed() != null) {
            MaxVehicleSpeed.setText(vehicul.getMaxVehicleSpeed());
        }
        if (vehicul.getPlayerStandOnTop() != null) {
            PlayerStandOnTop.setText(vehicul.getPlayerStandOnTop());
        }
        if (vehicul.getScaleModifier() != null) {
            ScaleModifier.setText(vehicul.getScaleModifier());
        }
        if (vehicul.getShapeYOffset() != null) {
            ShapeYOffset.setText(vehicul.getShapeYOffset());
        }
        if (vehicul.getCenterOfGravityOffset() != null) {
            CenterOfGravityOffset.setText(vehicul.getCenterOfGravityOffset());
        }

    }

    @FXML
    public void save(MouseEvent mouseEvent) {
    }
}
