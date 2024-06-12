package com.mk.cpm.controller;

import com.mk.cpm.HelloApplication;
import com.mk.cpm.config.Config;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.PasswordField;
import javafx.scene.control.TextField;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseEvent;
import javafx.stage.Stage;

import java.io.*;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.ResourceBundle;

public class LoginController implements Initializable {
    public static Scene s;

    @FXML
    public ImageView image;
    public Button login;
    public PasswordField pwd;
    public TextField user;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        //http://62.4.21.235/logo.png
        Image i = new Image("http://62.4.21.235/logo.png");
        image.setImage(i);


        if (Config.getUser() !=null) {
            user.setText(Config.getUser());
        }

    }

    @FXML
    public void login(ActionEvent actionEvent) throws IOException {
        Stage stage = (Stage) login.getScene().getWindow();
        Config.setUser(user.getText());
        Config.saveConfig();
        try {
            String username = user.getText();
            String password = pwd.getText();
            boolean authenticated = sendPOST(username, password);
            if (authenticated) {
                stage.close();
                System.out.println("Connexion réussie");
                FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("main.fxml"));
                Scene scene = new Scene(fxmlLoader.load());
                stage.getIcons().add(HelloApplication.logo);
                stage.setTitle("CPM-UI");
                stage.setScene(scene);
                stage.setResizable(false);
                stage.show();
                s = scene;
            } else {
                //mets le button en rouge
                login.setStyle("-fx-background-color: red");
                System.out.println("Nom d'utilisateur ou mot de passe incorrect");
            }
        } catch (IOException e) {
            e.printStackTrace();
        }
    }


    private static boolean sendPOST(String username, String password) throws IOException {
        String urlString = "http://62.4.21.235/test.php";
        URL obj = new URL(urlString);
        HttpURLConnection con = (HttpURLConnection) obj.openConnection();
        con.setRequestMethod("POST");
        con.setRequestProperty("Content-Type", "application/json; utf-8");
        con.setRequestProperty("Accept", "application/json");
        con.setDoOutput(true);

        String jsonInputString = String.format("{\"username\": \"%s\", \"password\": \"%s\"}", username, password);

        try (OutputStream os = con.getOutputStream()) {
            byte[] input = jsonInputString.getBytes("utf-8");
            os.write(input, 0, input.length);
        }

        int responseCode = con.getResponseCode();
        if (responseCode == HttpURLConnection.HTTP_OK) {
            BufferedReader in = new BufferedReader(new InputStreamReader(con.getInputStream(), "utf-8"));
            String inputLine;
            StringBuilder response = new StringBuilder();

            while ((inputLine = in.readLine()) != null) {
                response.append(inputLine);
            }
            in.close();

            // Analyser la réponse JSON
            String jsonResponse = response.toString();
            return jsonResponse.contains("\"authenticated\":true");
        } else {
            System.out.println("Network error: " + con);
            System.out.println("POST request not worked");
            return false;
        }
    }

    @FXML
    public void info(MouseEvent actionEvent) {
        Stage stage = new Stage();
        FXMLLoader fxmlLoader = new FXMLLoader(HelloApplication.class.getResource("info.fxml"));
        Scene scene = null;
        try {
            scene = new Scene(fxmlLoader.load());
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        stage.setTitle("Informations");
        stage.getIcons().add(HelloApplication.logo);
        stage.setScene(scene);
        stage.show();
    }
}
