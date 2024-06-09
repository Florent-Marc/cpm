package com.mk.cpm.loader.object;

import com.mk.cpm.loader.DataModifier;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

public class Vehicul extends Item implements DataModifier {

    private List<Wheel> Wheels;
    private List<Seat> Seats;
    private List<Shape> Shapes;
    private Item item;
    private String CenterOfGravityOffset;
    private String DragCoefficient;
    private String EmptyMass;
    private String AngularDamping;
    private String LinearDamping;
    private String DefaultEngine;
    private String DefaultSounds;
    private String DefaultZoomLevel;
    private String MaxVehicleSpeed;
    private String PlayerStandOnTop;
    private String ScaleModifier;
    private String ShapeYOffset;
    private List<String> infos;


    @Override
    public Object load(File file) {
        this.infos = new ArrayList<>();
        this.item = (Item) super.load(file);
        this.Seats = new ArrayList<>();
        for (String name : super.getSections(file, "Seat")) {
            Seat seat = new Seat();
            this.Seats.add((Seat) seat.load(file, name));
            super.getInfos().addAll(seat.getInfos());
        }
        this.Wheels = new ArrayList<>();
        for (String name : super.getSections(file, "Wheel")) {
            if (name.contains("Steering")) {
                continue;
            }
            Wheel wheel = new Wheel();
            this.Wheels.add((Wheel) wheel.load(file, name));
            super.getInfos().addAll(wheel.getInfos());
        }
        this.Shapes = new ArrayList<>();
        for (String name : super.getSections(file, "Shape")) {
            Shape shape = new Shape();
            shape.load(file, name);
            this.Shapes.add(shape);
            super.getInfos().addAll(shape.getInfos());
        }
        this.CenterOfGravityOffset = super.getValues(infos, file, "CenterOfGravityOffset");
        this.DragCoefficient = super.getValues(infos, file, "DragCoefficient");
        this.EmptyMass = super.getValues(infos, file, "EmptyMass");
        this.AngularDamping = super.getValues(infos, file, "AngularDamping");
        this.LinearDamping = super.getValues(infos, file, "LinearDamping");
        this.DefaultEngine = super.getValues(infos, file, "DefaultEngine");
        this.DefaultSounds = super.getValues(infos, file, "DefaultSounds");
        this.DefaultZoomLevel = super.getValues(infos, file, "DefaultZoomLevel");
        this.MaxVehicleSpeed = super.getValues(infos, file, "MaxVehicleSpeed");
        this.PlayerStandOnTop = super.getValues(infos, file, "PlayerStandOnTop");
        this.ScaleModifier = super.getValues(infos, file, "ScaleModifier");
        this.ShapeYOffset = super.getValues(infos, file, "ShapeYOffset");
        infos.addAll(super.getInfos());
        return this;
    }

    public List<Wheel> getWheels() {
        return Wheels;
    }

    public void setWheels(List<Wheel> wheels) {
        Wheels = wheels;
    }

    public List<Seat> getSeats() {
        return Seats;
    }

    public void setSeats(List<Seat> seats) {
        Seats = seats;
    }

    public List<Shape> getShapes() {
        return Shapes;
    }

    public void setShapes(List<Shape> shapes) {
        Shapes = shapes;
    }

    public Item getItem() {
        return item;
    }

    public void setItem(Item item) {
        this.item = item;
    }

    public String getCenterOfGravityOffset() {
        return CenterOfGravityOffset;
    }

    public void setCenterOfGravityOffset(String centerOfGravityOffset) {
        CenterOfGravityOffset = centerOfGravityOffset;
    }

    public String getDragCoefficient() {
        return DragCoefficient;
    }

    public void setDragCoefficient(String dragCoefficient) {
        DragCoefficient = dragCoefficient;
    }

    public String getEmptyMass() {
        return EmptyMass;
    }

    public void setEmptyMass(String emptyMass) {
        EmptyMass = emptyMass;
    }

    public String getAngularDamping() {
        return AngularDamping;
    }

    public void setAngularDamping(String angularDamping) {
        AngularDamping = angularDamping;
    }

    public String getLinearDamping() {
        return LinearDamping;
    }

    public void setLinearDamping(String linearDamping) {
        LinearDamping = linearDamping;
    }

    public String getDefaultEngine() {
        return DefaultEngine;
    }

    public void setDefaultEngine(String defaultEngine) {
        DefaultEngine = defaultEngine;
    }

    public String getDefaultSounds() {
        return DefaultSounds;
    }

    public void setDefaultSounds(String defaultSounds) {
        DefaultSounds = defaultSounds;
    }

    public String getDefaultZoomLevel() {
        return DefaultZoomLevel;
    }

    public void setDefaultZoomLevel(String defaultZoomLevel) {
        DefaultZoomLevel = defaultZoomLevel;
    }

    public String getMaxVehicleSpeed() {
        return MaxVehicleSpeed;
    }

    public void setMaxVehicleSpeed(String maxVehicleSpeed) {
        MaxVehicleSpeed = maxVehicleSpeed;
    }

    public String getPlayerStandOnTop() {
        return PlayerStandOnTop;
    }

    public void setPlayerStandOnTop(String playerStandOnTop) {
        PlayerStandOnTop = playerStandOnTop;
    }

    public String getScaleModifier() {
        return ScaleModifier;
    }

    public void setScaleModifier(String scaleModifier) {
        ScaleModifier = scaleModifier;
    }

    public String getShapeYOffset() {
        return ShapeYOffset;
    }

    public void setShapeYOffset(String shapeYOffset) {
        ShapeYOffset = shapeYOffset;
    }

    public void setInfos(List<String> infos) {
        this.infos = infos;
    }

    @Override
    public List<String> getInfos() {
        return this.infos;
    }
}
