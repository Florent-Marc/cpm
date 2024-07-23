package com.mk.cpm.loader;

import java.util.List;

public class MultiParam {

    private List<String> list;
    private String name;

    public MultiParam(List<String> list, String name) {
        this.list = list;
        this.name = name;
    }

    public List<String> getList() {
        return list;
    }

    public String getName() {
        return name;
    }

}
