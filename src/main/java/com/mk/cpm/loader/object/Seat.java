package com.mk.cpm.loader.object;

import com.mk.cpm.loader.utils.DataModifier;
import com.mk.cpm.loader.utils.DataUtils;

import java.io.File;
import java.io.FileWriter;
import java.util.ArrayList;
import java.util.List;

public class Seat extends DataUtils implements DataModifier{

    //obligatoroire
    private String Name;
    private String Position;
    private String Driver;
    //optionnel
    private String CameraPositionY;
    private String CameraRotation;
    private String DependsOn;
    private String LinkedDoorPart;
    private String MaxPitch;
    private String MaxYaw;
    private String MinPitch;
    private String MinYaw;
    private String PlayerPosition;
    private String PlayerSize;
    private String Rotation;
    private String Scale;
    private String ShouldLimitFieldOfView;
    private List<String> infos;


    public String getName() {
        return Name;
    }

    public void setName(String name) {
        Name = name;
    }

    public String getPosition() {
        return Position;
    }

    public void setPosition(String position) {
        Position = position;
    }

    public String getDriver() {
        return Driver;
    }

    public void setDriver(String driver) {
        Driver = driver;
    }

    public String getCameraPositionY() {
        return CameraPositionY;
    }

    public void setCameraPositionY(String cameraPositionY) {
        CameraPositionY = cameraPositionY;
    }

    public String getCameraRotation() {
        return CameraRotation;
    }

    public void setCameraRotation(String cameraRotation) {
        CameraRotation = cameraRotation;
    }

    public String getDependsOn() {
        return DependsOn;
    }

    public void setDependsOn(String dependsOn) {
        DependsOn = dependsOn;
    }

    public String getLinkedDoorPart() {
        return LinkedDoorPart;
    }

    public void setLinkedDoorPart(String linkedDoorPart) {
        LinkedDoorPart = linkedDoorPart;
    }

    public String getMaxPitch() {
        return MaxPitch;
    }

    public void setMaxPitch(String maxPitch) {
        MaxPitch = maxPitch;
    }

    public String getMaxYaw() {
        return MaxYaw;
    }

    public void setMaxYaw(String maxYaw) {
        MaxYaw = maxYaw;
    }

    public String getMinPitch() {
        return MinPitch;
    }

    public void setMinPitch(String minPitch) {
        MinPitch = minPitch;
    }

    public String getMinYaw() {
        return MinYaw;
    }

    public void setMinYaw(String minYaw) {
        MinYaw = minYaw;
    }

    public String getPlayerPosition() {
        return PlayerPosition;
    }

    public void setPlayerPosition(String playerPosition) {
        PlayerPosition = playerPosition;
    }

    public String getPlayerSize() {
        return PlayerSize;
    }

    public void setPlayerSize(String playerSize) {
        PlayerSize = playerSize;
    }

    public String getRotation() {
        return Rotation;
    }

    public void setRotation(String rotation) {
        Rotation = rotation;
    }

    public String getScale() {
        return Scale;
    }

    public void setScale(String scale) {
        Scale = scale;
    }

    public String getShouldLimitFieldOfView() {
        return ShouldLimitFieldOfView;
    }

    public void setShouldLimitFieldOfView(String shouldLimitFieldOfView) {
        ShouldLimitFieldOfView = shouldLimitFieldOfView;
    }


    public Object load(File file, String name) {
        this.infos = new ArrayList<>();
        this.Name = name;
        this.Position = getValuesFromSection(infos, file, name, "Position");
        this.Driver = getValuesFromSection(infos, file, name, "Driver:");
        this.CameraPositionY = getValuesFromSection(infos, file, name, "CameraPositionY");
        this.CameraRotation = getValuesFromSection(infos, file, name, "CameraRotation");
        this.DependsOn = getValuesFromSection(infos, file, name, "DependsOn");
        this.LinkedDoorPart = getValuesFromSection(infos, file, name, "LinkedDoorPart");
        this.MaxPitch = getValuesFromSection(infos, file, name, "MaxPitch");
        this.MaxYaw = getValuesFromSection(infos, file, name, "MaxYaw");
        this.MinPitch = getValuesFromSection(infos, file, name, "MinPitch");
        this.MinYaw = getValuesFromSection(infos, file, name, "MinYaw");
        this.PlayerPosition = getValuesFromSection(infos, file, name, "PlayerPosition");
        this.PlayerSize = getValuesFromSection(infos, file, name, "PlayerSize");
        this.Rotation = getValuesFromSection(infos, file, name, "Rotation");
        this.Scale = getValuesFromSection(infos, file, name, "Scale");
        this.ShouldLimitFieldOfView = getValuesFromSection(infos, file, name, "ShouldLimitFieldOfView");
        return this;
    }


    @Override
    public Object load(File file) {
        return null;
    }

    @Override
    public void save(File file) {

    }

    @Override
    public List<String> getInfos() {
        return this.infos;
    }
}
