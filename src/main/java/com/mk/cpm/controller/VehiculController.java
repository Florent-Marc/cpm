package com.mk.cpm.controller;

import com.interactivemesh.jfx.importer.obj.ObjModelImporter;
import com.mk.cpm.HelloApplication;
import com.mk.cpm.AppMain;
import com.mk.cpm.config.Config;
import com.mk.cpm.loader.Loader;
import com.mk.cpm.loader.object.Seat;
import com.mk.cpm.loader.object.Shape;
import com.mk.cpm.loader.object.Vehicul;
import com.mk.cpm.loader.object.Wheel;
import com.mk.cpm.loader.pack.ZipCompressor;
import javafx.animation.KeyFrame;
import javafx.animation.KeyValue;
import javafx.animation.Timeline;
import javafx.beans.value.ChangeListener;
import javafx.beans.value.ObservableValue;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.*;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ChoiceBox;
import javafx.scene.control.Slider;
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
import java.util.ArrayList;
import java.util.List;
import java.util.ResourceBundle;

import static com.mk.cpm.HelloApplication.logo;

public class VehiculController implements Initializable {

    public ChoiceBox seatlist;
    public ChoiceBox shapelist;
    public Pane render;
    public Slider slider;
    private Vehicul vehicul;
    public static VehiculController instance;

    @FXML
    //item
    public TextField CreativeTab;
    public Slider zoom;
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
    public ChoiceBox wheelist;
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
    public CheckBox IsRight;
    public CheckBox HandBrakingWheel;
    //shape
    public TextField Names;
    public TextField Scales;
    public TextField Positions;
    //seat
    public CheckBox IsDriver;
    public TextField nameseat;
    public TextField Positionseat;
    public TextField CameraPositionY;
    public TextField CameraRotation;
    public TextField DependsOnseat;
    public TextField LinkedDoorPart;
    public TextField MaxPitch;
    public TextField MaxYaw;
    public TextField MinPitch;
    public TextField MinYaw;
    public TextField PlayerPosition;
    public TextField PlayerSize;
    public TextField Rotation;
    public TextField Scaleseat;
    public TextField ShouldLimitFieldOfView;

    private static int zoomValue = -10;


    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        if (Main.objectSelect == null) {
            return;
        }
        instance = this;
        vehicul = (Vehicul) Main.objectSelect;
        if (vehicul == null) {
            return;
        }

        com.mk.cpm.Main.o = vehicul;
        if (vehicul.getName() != null) {
            name.setText(vehicul.getName());
        }
        if (vehicul.getDesc() != null) {
            desc.setText(vehicul.getDesc());
        }
        if (vehicul.getModel() != null) {
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

        if (vehicul.getWheels() != null) {
            List<String> wheels = new ArrayList<>();
            for (int i = 0; i < vehicul.getWheels().size(); i++) {
                wheels.add(vehicul.getWheels().get(i).getName());
            }
            wheelist.getItems().addAll(wheels);
            wheelist.getSelectionModel().selectFirst();
        }
        if (wheelist.getSelectionModel() != null) {
            Wheel wheel = vehicul.getWheels().get(wheelist.getSelectionModel().getSelectedIndex());
            if (wheel.getAttachedWheel() != null) {
                AttachedWheel.setText(wheel.getAttachedWheel());
            }
            if (wheel.getDrivingWheel() != null) {
                DrivingWheel.setText(wheel.getDrivingWheel());
            }
            if (wheel.getIsSteerable() != null) {
                IsSteerable.setText(wheel.getIsSteerable());
            }
            if (wheel.getMaxTurn() != null) {
                MaxTurn.setText(wheel.getMaxTurn());
            }
            if (wheel.getPosition() != null) {
                Position.setText(wheel.getPosition());
            }
            if (wheel.getDependsOn() != null) {
                DependsOn.setText(wheel.getDependsOn());
            }
            if (wheel.getMudGuard() != null) {
                MudGuard.setText(wheel.getMudGuard());
            }
            if (wheel.getRotationPoint() != null) {
                RotationPoint.setText(wheel.getRotationPoint());
            }
            if (wheel.getScale() != null) {
                Scale.setText(wheel.getScale());
            }
            if (wheel.getSuspension() != null) {
                Suspension.setText(wheel.getSuspension());
            }
            if (wheel.getIsRight() != null) {
                IsRight.setSelected(Boolean.parseBoolean(wheel.getIsRight()));
            }
            if (wheel.getHandBrakingWheel() != null) {
                HandBrakingWheel.setSelected(Boolean.parseBoolean(wheel.getHandBrakingWheel()));
            }

        }

        if (vehicul.getShapes() != null) {
            List<String> shapes = new ArrayList<>();
            for (int i = 0; i < vehicul.getShapes().size(); i++) {
                shapes.add(vehicul.getShapes().get(i).getName());
            }
            shapelist.getItems().addAll(shapes);
            shapelist.getSelectionModel().selectFirst();
        }
        if (shapelist.getSelectionModel() != null) {
            Shape shape = vehicul.getShapes().get(shapelist.getSelectionModel().getSelectedIndex());
            if (shape.getName() != null) {
                Names.setText(shape.getName());
            }
            if (shape.getPosition() != null) {
                Positions.setText(shape.getPosition());
            }
            if (shape.getScale() != null) {
                Scales.setText(shape.getScale());
            }
        }

        if (vehicul.getSeats() != null) {
            List<String> seats = new ArrayList<>();
            for (int i = 0; i < vehicul.getSeats().size(); i++) {
                seats.add(vehicul.getSeats().get(i).getName());
            }
            seatlist.getItems().addAll(seats);
            seatlist.getSelectionModel().selectFirst();
        }
        if (seatlist.getSelectionModel() != null) {
            Seat seat = vehicul.getSeats().get(seatlist.getSelectionModel().getSelectedIndex());
            if (seat.getName() != null) {
                nameseat.setText(seat.getName());
            }
            if (seat.getPosition() != null) {
                Positionseat.setText(seat.getPosition());
            }
            if (seat.getCameraPositionY() != null) {
                CameraPositionY.setText(seat.getCameraPositionY());
            }
            if (seat.getCameraRotation() != null) {
                CameraRotation.setText(seat.getCameraRotation());
            }
            if (seat.getDependsOn() != null) {
                DependsOnseat.setText(seat.getDependsOn());
            }
            if (seat.getLinkedDoorPart() != null) {
                LinkedDoorPart.setText(seat.getLinkedDoorPart());
            }
            if (seat.getMaxPitch() != null) {
                MaxPitch.setText(seat.getMaxPitch());
            }
            if (seat.getMaxYaw() != null) {
                MaxYaw.setText(seat.getMaxYaw());
            }
            if (seat.getMinPitch() != null) {
                MinPitch.setText(seat.getMinPitch());
            }
            if (seat.getMinYaw() != null) {
                MinYaw.setText(seat.getMinYaw());
            }
            if (seat.getPlayerPosition() != null) {
                PlayerPosition.setText(seat.getPlayerPosition());
            }
            if (seat.getPlayerSize() != null) {
                PlayerSize.setText(seat.getPlayerSize());
            }
            if (seat.getRotation() != null) {
                Rotation.setText(seat.getRotation());
            }
            if (seat.getScale() != null) {
                Scaleseat.setText(seat.getScale());
            }
            if (seat.getShouldLimitFieldOfView() != null) {
                ShouldLimitFieldOfView.setText(seat.getShouldLimitFieldOfView());
            }
            if (seat.getDriver() != null) {
                IsDriver.setSelected(Boolean.parseBoolean(seat.getDriver()));
            }
        }

        ObjModelImporter myModel = new ObjModelImporter();
        try {
            String[] split = vehicul.getModel().split("/");
            File file = new File(Loader.getPath(split[split.length - 1], Main.packSelected));
            if (file.exists() && file.getName().contains(".obj")) {
                myModel.read(file);
            }
        } catch (Exception e) {
            System.out.println("Error while loading model");
            return;
        }

        MeshView[] meshViews = myModel.getImport();
        if (meshViews == null || meshViews.length == 0) {
            return;
        }

        Group root = new Group(meshViews);


        for (MeshView meshView : meshViews) {
            PhongMaterial material = new PhongMaterial();
            material.setDiffuseColor(Color.color(Math.random(), Math.random(), Math.random()));
            meshView.setMaterial(material);
        }

        Camera camera = new PerspectiveCamera(true);
        slider.valueProperty().addListener(new ChangeListener<Number>() {
            @Override
            public void changed(ObservableValue<? extends Number> observable, Number oldValue, Number newValue) {
                // Update the label with the new slider value
                camera.setTranslateZ(-newValue.doubleValue());
            }
        });
        camera.setTranslateZ(zoomValue);

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
        subScene.setStyle("-fx-background-color:  #24305a;");
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

        zoom = new Slider();
        zoom.setMin(0);
        zoom.setMax(100);
        zoom.setValue(10);
        zoom.setShowTickLabels(true);
        zoom.setShowTickMarks(true);
        zoom.setMajorTickUnit(10);
        zoom.setMinorTickCount(5);
        zoom.setBlockIncrement(10);


        root.getChildren().add(zoom);
    }

    @FXML
    public void choiceWheel(ActionEvent actionEvent) {
        Wheel wheel = vehicul.getWheels().get(wheelist.getSelectionModel().getSelectedIndex());
        if (wheel.getAttachedWheel() != null) {
            AttachedWheel.setText(wheel.getAttachedWheel());
        }
        if (wheel.getDrivingWheel() != null) {
            DrivingWheel.setText(wheel.getDrivingWheel());
        }
        if (wheel.getIsSteerable() != null) {
            IsSteerable.setText(wheel.getIsSteerable());
        }
        if (wheel.getMaxTurn() != null) {
            MaxTurn.setText(wheel.getMaxTurn());
        }
        if (wheel.getPosition() != null) {
            Position.setText(wheel.getPosition());
        }
        if (wheel.getDependsOn() != null) {
            DependsOn.setText(wheel.getDependsOn());
        }
        if (wheel.getMudGuard() != null) {
            MudGuard.setText(wheel.getMudGuard());
        }
        if (wheel.getRotationPoint() != null) {
            RotationPoint.setText(wheel.getRotationPoint());
        }
        if (wheel.getScale() != null) {
            Scale.setText(wheel.getScale());
        }
        if (wheel.getSuspension() != null) {
            Suspension.setText(wheel.getSuspension());
        }
        if (wheel.getIsRight() != null) {
            IsRight.setSelected(Boolean.parseBoolean(wheel.getIsRight()));
        }
        if (wheel.getHandBrakingWheel() != null) {
            HandBrakingWheel.setSelected(Boolean.parseBoolean(wheel.getHandBrakingWheel()));
        }
    }

    @FXML
    public void choicelist(ActionEvent actionEvent) {
        Shape shape = vehicul.getShapes().get(shapelist.getSelectionModel().getSelectedIndex());
        if (shape.getName() != null) {
            Names.setText(shape.getName());
        }
        if (shape.getPosition() != null) {
            Positions.setText(shape.getPosition());
        }
        if (shape.getScale() != null) {
            Scales.setText(shape.getScale());
        }
    }

    @FXML
    public void saveshape(ActionEvent actionEvent) {
        Shape shape = vehicul.getShapes().get(shapelist.getSelectionModel().getSelectedIndex());
        shape.setName(Names.getText());
        shape.setPosition(Positions.getText());
        shape.setScale(Scales.getText());
    }

    @FXML
    public void savewheel(ActionEvent actionEvent) {
        Wheel wheel = vehicul.getWheels().get(wheelist.getSelectionModel().getSelectedIndex());
        wheel.setAttachedWheel(AttachedWheel.getText());
        wheel.setDrivingWheel(DrivingWheel.getText());
        wheel.setIsSteerable(IsSteerable.getText());
        wheel.setMaxTurn(MaxTurn.getText());
        wheel.setPosition(Position.getText());
        wheel.setDependsOn(DependsOn.getText());
        wheel.setMudGuard(MudGuard.getText());
        wheel.setRotationPoint(RotationPoint.getText());
        wheel.setScale(Scale.getText());
        wheel.setSuspension(Suspension.getText());
        wheel.setIsRight(String.valueOf(IsRight.isSelected()));
        wheel.setHandBrakingWheel(String.valueOf(HandBrakingWheel.isSelected()));
    }

    @FXML
    public void saveSeat(ActionEvent actionEvent) {
        Seat seat = vehicul.getSeats().get(seatlist.getSelectionModel().getSelectedIndex());
        seat.setName(nameseat.getText());
        seat.setPosition(Positionseat.getText());
        seat.setCameraPositionY(CameraPositionY.getText());
        seat.setCameraRotation(CameraRotation.getText());
        seat.setDependsOn(DependsOnseat.getText());
        seat.setLinkedDoorPart(LinkedDoorPart.getText());
        seat.setMaxPitch(MaxPitch.getText());
        seat.setMaxYaw(MaxYaw.getText());
        seat.setMinPitch(MinPitch.getText());
        seat.setMinYaw(MinYaw.getText());
        seat.setPlayerPosition(PlayerPosition.getText());
        seat.setPlayerSize(PlayerSize.getText());
        seat.setRotation(Rotation.getText());
        seat.setScale(Scaleseat.getText());
        seat.setShouldLimitFieldOfView(ShouldLimitFieldOfView.getText());
        seat.setDriver(String.valueOf(IsDriver.isSelected()));
    }

    @FXML
    public void save(MouseEvent mouseEvent) {
        vehicul.setName(name.getText());
        vehicul.setDesc(desc.getText());
        vehicul.setModel(model.getText());
        vehicul.setCreativeTab(CreativeTab.getText());
        vehicul.setIconText(IconText.getText());
        vehicul.setItemRotation(itemrotation.getText());
        vehicul.getItem().setItemTranslate(itemtranslation.getText());
        vehicul.setItemScale(itemscale.getText());
        vehicul.setDragCoefficient(DragCoefficient.getText());
        vehicul.setEmptyMass(EmptyMass.getText());
        vehicul.setAngularDamping(AngularDamping.getText());
        vehicul.setLinearDamping(LinearDamping.getText());
        vehicul.setDefaultEngine(DefaultEngine.getText());
        vehicul.setDefaultSounds(DefaultSounds.getText());
        vehicul.setDefaultZoomLevel(DefaultZoomLevel.getText());
        vehicul.setMaxVehicleSpeed(MaxVehicleSpeed.getText());
        vehicul.setPlayerStandOnTop(PlayerStandOnTop.getText());
        vehicul.setScaleModifier(ScaleModifier.getText());
        vehicul.setShapeYOffset(ShapeYOffset.getText());
        vehicul.setCenterOfGravityOffset(CenterOfGravityOffset.getText());

        File f = vehicul.getFile();
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
        this.vehicul.save(f);
        if (f.getPath().contains("\\cpm\\cache")) {
            try {
                String source = Config.getCachePath() + "/pack/" + Main.packSelected;
                String dest = Config.getLastdirectory() + "/" + Main.packSelected;
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

    @FXML
    public void seatChoice(ActionEvent actionEvent) {
        Seat seat = vehicul.getSeats().get(seatlist.getSelectionModel().getSelectedIndex());
        if (seat.getName() != null) {
            nameseat.setText(seat.getName());
        }
        if (seat.getPosition() != null) {
            Positionseat.setText(seat.getPosition());
        }
        if (seat.getCameraPositionY() != null) {
            CameraPositionY.setText(seat.getCameraPositionY());
        }
        if (seat.getCameraRotation() != null) {
            CameraRotation.setText(seat.getCameraRotation());
        }
        if (seat.getDependsOn() != null) {
            DependsOnseat.setText(seat.getDependsOn());
        }
        if (seat.getLinkedDoorPart() != null) {
            LinkedDoorPart.setText(seat.getLinkedDoorPart());
        }
        if (seat.getMaxPitch() != null) {
            MaxPitch.setText(seat.getMaxPitch());
        }
        if (seat.getMaxYaw() != null) {
            MaxYaw.setText(seat.getMaxYaw());
        }
        if (seat.getMinPitch() != null) {
            MinPitch.setText(seat.getMinPitch());
        }
        if (seat.getMinYaw() != null) {
            MinYaw.setText(seat.getMinYaw());
        }
        if (seat.getPlayerPosition() != null) {
            PlayerPosition.setText(seat.getPlayerPosition());
        }
        if (seat.getPlayerSize() != null) {
            PlayerSize.setText(seat.getPlayerSize());
        }
        if (seat.getRotation() != null) {
            Rotation.setText(seat.getRotation());
        }
        if (seat.getScale() != null) {
            Scaleseat.setText(seat.getScale());
        }
        if (seat.getShouldLimitFieldOfView() != null) {
            ShouldLimitFieldOfView.setText(seat.getShouldLimitFieldOfView());
        }
        if (seat.getDriver() != null) {
            IsDriver.setSelected(Boolean.parseBoolean(seat.getDriver()));
        }
    }

    @FXML
    public void addseat(ActionEvent actionEvent) throws IOException {
        /*
        Stage stage = new Stage();
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("more.fxml"));
        Scene scene = new Scene(fxmlLoader.load());
        stage.getIcons().add(HelloApplication.logo);
        scene.setFill(Color.rgb(40, 44, 52, 1.0));
        stage.setTitle("CPM-UI");
        stage.setScene(scene);
        stage.setResizable(false);
        stage.show();

         */
        Seat s = new Seat();
        s.setName("Seat_" + vehicul.getSeats().size() + 1 + "{");
        vehicul.getSeats().add(s);
        seatlist.getItems().add(s.getName());
    }

    public void update(Vehicul vehicul) {
        this.vehicul = vehicul;
        shapelist.getItems().clear();
        seatlist.getItems().clear();
        wheelist.getItems().clear();
        if (vehicul.getShapes() != null) {
            List<String> shapes = new ArrayList<>();
            for (int i = 0; i < vehicul.getShapes().size(); i++) {
                shapes.add(vehicul.getShapes().get(i).getName());
            }
            shapelist.getItems().addAll(shapes);
            shapelist.getSelectionModel().selectFirst();
        }
        if (vehicul.getSeats() != null) {
            List<String> seats = new ArrayList<>();
            for (int i = 0; i < vehicul.getSeats().size(); i++) {
                seats.add(vehicul.getSeats().get(i).getName());
            }
            seatlist.getItems().addAll(seats);
            seatlist.getSelectionModel().selectFirst();
        }
        if (vehicul.getWheels() != null) {
            List<String> wheels = new ArrayList<>();
            for (int i = 0; i < vehicul.getWheels().size(); i++) {
                wheels.add(vehicul.getWheels().get(i).getName());
            }
            wheelist.getItems().addAll(wheels);
            wheelist.getSelectionModel().selectFirst();
        }
    }


    @FXML
    public void addon(MouseEvent mouseEvent) {
        Stage fd = new Stage();
        FXMLLoader jk = new FXMLLoader(HelloApplication.class.getResource("listaddon.fxml"));
        Scene d = null;
        try {
            d = new Scene(jk.load());
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        fd.getIcons().add(logo);
        fd.setTitle("CPM-UI");
        fd.setScene(d);
        fd.setResizable(false);
        fd.show();
    }
}
