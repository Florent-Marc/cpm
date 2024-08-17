module com.mk.cpm {
    requires javafx.controls;
    requires javafx.fxml;
    requires javafx.web;

    requires org.controlsfx.controls;
    requires eu.hansolo.tilesfx;
    requires json.simple;
    requires jimObjModelImporterJFX;
    requires org.apache.httpcomponents.httpclient;
    requires org.apache.httpcomponents.httpcore;
    requires org.json;
    requires org.javassist;
    requires imgui.app;

    opens com.mk.cpm to javafx.fxml;
    exports com.mk.cpm;
    exports com.mk.cpm.controller;
    exports com.mk.cpm.config;
    exports com.mk.cpm.converter;
    exports com.mk.cpm.loader.object;
    opens com.mk.cpm.controller to javafx.fxml;
}