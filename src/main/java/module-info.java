module com.mk.cpm {
    requires javafx.controls;
    requires javafx.fxml;
    requires javafx.web;

    requires org.controlsfx.controls;
    requires eu.hansolo.tilesfx;
    requires json.simple;
    requires jimObjModelImporterJFX;

    opens com.mk.cpm to javafx.fxml;
    exports com.mk.cpm;
    exports com.mk.cpm.controller;
    opens com.mk.cpm.controller to javafx.fxml;
}