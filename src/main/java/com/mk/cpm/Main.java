package com.mk.cpm;

import com.mk.cpm.config.Config;

public class Main {
    public static void main(String[] args) {
        Config.load();
        HelloApplication.main(args);
    }
}
