#include <Arduino.h>
#include <WiFi.h>

const char* ssid     = "ESP32_AP";
const char* password = NULL;
const IPAddress local_ip(192, 168, 0, 2);
const IPAddress gateway(192, 168, 0, 1);
const IPAddress subnet(255, 255, 255, 0);

void setup() {
    Serial.begin(115200);
    Serial.println("\n[*] Creating AP");
    WiFi.mode(WIFI_AP);
    WiFi.softAP(ssid, password);
    WiFi.softAPConfig(local_ip, gateway, subnet);
    Serial.print("[+] AP Created with local IP ");
    Serial.println(WiFi.softAPIP());
}

void loop() {}