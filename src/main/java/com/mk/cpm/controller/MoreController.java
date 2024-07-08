package com.mk.cpm.controller;

import com.mk.cpm.Main;
import com.mk.cpm.loader.object.Seat;
import com.mk.cpm.loader.object.Vehicul;
import javafx.fxml.FXML;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;

public class MoreController {

    @FXML
    public TextField name;

    @FXML
    public void create(MouseEvent mouseEvent) {
        Vehicul v = (Vehicul) Main.o;
        String cat = Main.CurrentAdd;
        if (cat.equals("seat")) {
            Seat s = new Seat();
            s.setName("Seat_"+name.getText()+"{");
            v.getSeats().add(s);
        }
        if (cat.equals("wheel")) {
            Seat s = new Seat();
            s.setName("Wheel_"+name.getText()+"{");
            v.getSeats().add(s);
        }
        if (cat.equals("shape")) {
            Seat s = new Seat();
            s.setName("Shape_"+name.getText()+"{");
            v.getSeats().add(s);
        }

        //VehiculController.instance.update(v);
    }


}
