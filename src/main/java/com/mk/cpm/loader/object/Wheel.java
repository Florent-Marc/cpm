package com.mk.cpm.loader.object;

import com.mk.cpm.loader.utils.DataModifier;
import com.mk.cpm.loader.utils.DataUtils;

import java.io.File;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

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
    public Object load(File file) {
        return null;
    }

    @Override
    public void save(File file) {
        Map<String, String> map = new HashMap<>();
        map.put("AttachedWheel", AttachedWheel);
        map.put("DrivingWheel", DrivingWheel);
        map.put("IsRight", IsRight);
        map.put("IsSteerable", IsSteerable);
        map.put("MaxTurn", MaxTurn);
        map.put("Position", Position);
        map.put("DependsOn", DependsOn);
        map.put("HandBrakingWheel", HandBrakingWheel);
        map.put("MudGuard", MudGuard);
        map.put("RotationPoint", RotationPoint);
        map.put("Scale", Scale);
        map.put("Suspension", Suspension);
        setMultiValues(file, Name, map);
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

    public String getName() {
        return Name;
    }

    public void setName(String name) {
        Name = name;
    }

    public void setInfos(List<String> infos) {
        this.infos = infos;
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
