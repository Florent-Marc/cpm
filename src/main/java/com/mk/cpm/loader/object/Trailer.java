package com.mk.cpm.loader.object;

import java.util.ArrayList;
import java.util.List;

public class Trailer extends Item{

    private String AttachPoint;
    private String AttachStrength;

    public String getAttachPoint() {
        return AttachPoint;
    }

    public void setAttachPoint(String attachPoint) {
        AttachPoint = attachPoint;
    }

    public String getAttachStrength() {
        return AttachStrength;
    }

    public void setAttachStrength(String attachStrength) {
        AttachStrength = attachStrength;
    }

    @Override
    public List<String> getInfos() {
        return new ArrayList<>();
    }
}
