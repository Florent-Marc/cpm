package com.mk.cpm;

import org.apache.http.client.methods.CloseableHttpResponse;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.impl.client.HttpClients;
import org.apache.http.util.EntityUtils;
import org.json.JSONArray;
import org.json.JSONObject;

import java.io.BufferedInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.URL;

public class GitHubReleasesFetcher {
    public static double newVersion = 0.0;
    public static String name = "";
    public static void main(String[] args) {
        String url = "https://api.github.com/repos/Florent-Marc/cpm/releases";
        try (CloseableHttpClient httpClient = HttpClients.createDefault()) {
            HttpGet request = new HttpGet(url);
            try (CloseableHttpResponse response = httpClient.execute(request)) {
                String jsonResponse = EntityUtils.toString(response.getEntity());
                JSONArray releases = new JSONArray(jsonResponse);
                for (int i = 0; i < releases.length(); i++) {
                    JSONObject release = releases.getJSONObject(i);
                    String name = release.getString("name");
                    String publishedAt = release.getString("published_at");
                    System.out.println("Release: " + name);
                    System.out.println("Published at: " + publishedAt);

                        //get le chemin du programme lancé
                        String path = System.getProperty("user.dir");
                        System.out.println("Current path: " + path);
                        //System.out.println("New version available: " + version);
                        //downloadFile("https://github.com/Florent-Marc/cpm/releases/download/"+name.replace(" ","-")+"/CPM-UI.jar", "C:\\Users\\chouc\\Downloads\\CPM-UI.jar");


                    System.out.println("---------------------------");
                    // remove the first word
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public static void downloadFile(String fileURL, String savePath) throws IOException {
        try (InputStream in = new URL(fileURL).openStream();
             BufferedInputStream bis = new BufferedInputStream(in);
             FileOutputStream fis = new FileOutputStream(savePath)) {
            byte[] buffer = new byte[1024];
            int count = 0;
            while ((count = bis.read(buffer, 0, 1024)) != -1) {
                fis.write(buffer, 0, count);
            }
        }
    }

    //methode for check if new version is available
    public static boolean isNewVersionAvailable(double version){
        String url = "https://api.github.com/repos/Florent-Marc/cpm/releases";
        try (CloseableHttpClient httpClient = HttpClients.createDefault()) {
            HttpGet request = new HttpGet(url);
            try (CloseableHttpResponse response = httpClient.execute(request)) {
                String jsonResponse = EntityUtils.toString(response.getEntity());
                JSONArray releases = new JSONArray(jsonResponse);
                for (int i = 0; i < releases.length(); i++) {
                    JSONObject release = releases.getJSONObject(i);
                    String n = release.getString("name");
                    String publishedAt = release.getString("published_at");
                    //get le texte apres le premier espace
                    double v = Double.parseDouble(n.substring(n.indexOf(" ")+1));
                    if(v > version){
                        newVersion = v;
                        name = n;
                        return true;
                    }
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        return false;
    }

}

