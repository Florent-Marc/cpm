package com.mk.cpm;

import com.jagrosh.discordipc.IPCClient;
import com.jagrosh.discordipc.IPCListener;
import com.jagrosh.discordipc.entities.RichPresence;
import com.jagrosh.discordipc.entities.RichPresenceButton;
import com.jagrosh.discordipc.entities.pipe.PipeStatus;
import com.jagrosh.discordipc.exceptions.NoDiscordClientException;

public class DiscordRPC {

    public static long currentTime = System.currentTimeMillis();
    public static IPCClient client;

    public static void initRPC() {
        client = new IPCClient(1250910987381510295L);
        client.setListener(new IPCListener() {
            @Override
            public void onReady(IPCClient client) {}
        });
        try {
            if(!client.getStatus().equals(PipeStatus.CONNECTED)){
                client.connect();
            }
        } catch (NoDiscordClientException e) {
            e.printStackTrace();
        }

        new Thread("RPC-Callback-Handler") {
            @Override
            public void run() {
                while (!Thread.currentThread().isInterrupted()) {
                    try {
                        RichPresenceButton firstDiscordButton = new RichPresenceButton("cmd /c start https://discord.gg/7USXZZTf4h", "Discord");
                        RichPresenceButton[] discordButton = new RichPresenceButton[]{firstDiscordButton};
                        DiscordRPC.updatePresence("Menu de connexion", "", "","", "", "", discordButton);
                        Thread.sleep(2000);
                    } catch (InterruptedException ignored) {
                        client.close();
                    }
                }
            }
        }.start();

    }
    public static void updatePresence(String details, String state, String largeImageKey, String largeImageText, String smallImageKey, String smallImageText, RichPresenceButton[] button) {
        if(client == null) return;
        RichPresence.Builder builder = new RichPresence.Builder();
        builder.setState(state)
                .setDetails(details)
                .setStartTimestamp(currentTime)
                .setLargeImage(largeImageKey, largeImageText)
                .setSmallImage(smallImageKey, smallImageText)
                .setButtons(button);
        if(client.getStatus().equals(PipeStatus.CONNECTED)) {
            client.sendRichPresence(builder.build());
        }
    }
}
