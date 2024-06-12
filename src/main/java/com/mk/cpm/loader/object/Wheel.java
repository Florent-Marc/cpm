package com.mk.cpm.loader.object;

import com.mk.cpm.loader.utils.DataModifier;
import com.mk.cpm.loader.utils.DataUtils;

import java.io.File;
import java.io.FileWriter;
import java.util.ArrayList;
import java.util.List;

public class Wheel extends DataUtils implements DataModifier {

    //obligatory
    private String Name;
    private String AttachedWheel;
    private String DrivingWheel;
    private String IsRight;
    private String IsSteerable;
    private String MaxTurn;
    private String Position;
    //optional
    private String DependsOn;
    private String HandBrakingWheel;
    private String MudGuard;
    private String RotationPoint;
    private String Scale;
    private String Suspension;
    private List<String> infos;

    @Override
    public Object load(File br) {
        return null;
    }

    @Override
    public void save(FileWriter file) {
        setValues(file, "AttachedWheel", AttachedWheel);
        setValues(file, "DrivingWheel", DrivingWheel);
        setValues(file, "IsRight", IsRight);
        setValues(file, "IsSteerable", IsSteerable);
        setValues(file, "MaxTurn", MaxTurn);
        setValues(file, "Position", Position);
        setValues(file, "DependsOn", DependsOn);
        setValues(file, "HandBrakingWheel", HandBrakingWheel);
        setValues(file, "MudGuard", MudGuard);
        setValues(file, "RotationPoint", RotationPoint);
        setValues(file, "Scale", Scale);
        setValues(file, "Suspension", Suspension);
        /*
        Map<String, String> map = Map.of("AttachedWheel", AttachedWheel, "DrivingWheel", DrivingWheel);
        setMultiValues(file, "test",map);

         */
    }

    @Override
    public List<String> getInfos() {
        return this.infos;
    }

    public String getAttachedWheel() {
        return AttachedWheel;
    }

    public void setAttachedWheel(String attachedWheel) {
        AttachedWheel = attachedWheel;
    }

    public String getDrivingWheel() {
        return DrivingWheel;
    }

    public void setDrivingWheel(String drivingWheel) {
        DrivingWheel = drivingWheel;
    }

    public String getIsRight() {
        return IsRight;
    }

    public void setIsRight(String isRight) {
        IsRight = isRight;
    }

    public String getIsSteerable() {
        return IsSteerable;
    }

    public void setIsSteerable(String isSteerable) {
        IsSteerable = isSteerable;
    }

    public String getMaxTurn() {
        return MaxTurn;
    }

    public void setMaxTurn(String maxTurn) {
        MaxTurn = maxTurn;
    }

    public String getPosition() {
        return Position;
    }

    public void setPosition(String position) {
        Position = position;
    }

    public String getDependsOn() {
        return DependsOn;
    }

    public void setDependsOn(String dependsOn) {
        DependsOn = dependsOn;
    }

    public String getHandBrakingWheel() {
        return HandBrakingWheel;
    }

    public void setHandBrakingWheel(String handBrakingWheel) {
        HandBrakingWheel = handBrakingWheel;
    }

    public String getMudGuard() {
        return MudGuard;
    }

    public void setMudGuard(String mudGuard) {
        MudGuard = mudGuard;
    }

    public String getRotationPoint() {
        return RotationPoint;
    }

    public void setRotationPoint(String rotationPoint) {
        RotationPoint = rotationPoint;
    }

    public String getScale() {
        return Scale;
    }

    public void setScale(String scale) {
        Scale = scale;
    }

    public String getSuspension() {
        return Suspension;
    }

    public void setSuspension(String suspension) {
        Suspension = suspension;
    }

    public Object load(File file, String name) {
        this.infos = new ArrayList<>();
        this.Name = name;
        this.AttachedWheel = getValuesFromSection(infos, file, name, "AttachedWheel");
        this.DrivingWheel = getValuesFromSection(infos, file, name, "DrivingWheel");
        this.IsRight = getValuesFromSection(infos, file, name, "IsRight");
        this.IsSteerable = getValuesFromSection(infos, file, name, "IsSteerable");
        this.MaxTurn = getValuesFromSection(infos, file, name, "MaxTurn");
        this.Position = getValuesFromSection(infos, file, name, "Position");
        this.DependsOn = getValuesFromSection(infos, file, name, "DependsOn");
        this.HandBrakingWheel = getValuesFromSection(infos, file, name, "HandBrakingWheel");
        this.MudGuard = getValuesFromSection(infos, file, name, "MudGuard");
        this.RotationPoint = getValuesFromSection(infos, file, name, "RotationPoint");
        this.Scale = getValuesFromSection(infos, file, name, "Scale");
        this.Suspension = getValuesFromSection(infos, file, name, "Suspension");
        return this;
    }
}
