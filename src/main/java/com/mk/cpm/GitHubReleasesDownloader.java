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

public class GitHubReleasesDownloader {

    public static void main(String[] args) {
        String repoOwner = "Florent-Marc";
        String repoName = "cpm";
        String apiUrl = "https://api.github.com/repos/" + repoOwner + "/" + repoName + "/releases";

        try (CloseableHttpClient httpClient = HttpClients.createDefault()) {
            HttpGet request = new HttpGet(apiUrl);
            try (CloseableHttpResponse response = httpClient.execute(request)) {
                String jsonResponse = EntityUtils.toString(response.getEntity());
                JSONObject latestRelease = new JSONObject(jsonResponse);
                JSONArray assets = latestRelease.getJSONArray("assets");

                for (int i = 0; i < assets.length(); i++) {
                    JSONObject asset = assets.getJSONObject(i);
                    String assetName = asset.getString("name");
                    String downloadUrl = asset.getString("browser_download_url");

                    System.out.println("Downloading: " + assetName);
                    downloadFile(downloadUrl, "downloads/" + assetName);
                    System.out.println("Downloaded: " + assetName);
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
}
