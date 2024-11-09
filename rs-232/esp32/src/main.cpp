#include <Arduino.h>
#include <Wifi.h>
#include <LittleFS.h>
#include <ESPAsyncWebServer.h>
#include <ESPmDNS.h>

#define RXD2 16
#define TXD2 17
#define TIMEOUT 5       // Components startup timeout

const String AP_NAME = "ESP32_AP";
const String mDNS_NAME = "ESP32";

AsyncWebServer server(80);
AsyncWebSocket ws("/ws");

String msg = "";        // Holds a WebSocket message

String uart_buf;

unsigned long base_delay = 1000;
unsigned long counter = 0;
unsigned long start_time = 0;
unsigned long proc_time = 0;

void InitWebServer();
void InitWebSocket();
template <typename F>
bool InitComponent(const char* componentName, F InitFunction);
void OnEvent(AsyncWebSocket* server, AsyncWebSocketClient* client, AwsEventType type, void* arg, uint8_t* data, size_t len);
void HandleWebSocketMessage(void* arg, uint8_t* data, size_t len);
void NotFound(AsyncWebServerRequest *request);

void setup() {
    Serial.begin(9600);
    Serial2.begin(9600, SERIAL_8N1, RXD2, TXD2);

    WiFi.softAP(AP_NAME);
    Serial.println(WiFi.softAPIP());
    
    // Start LittleFS
    if (!InitComponent("LittleFS", []() -> bool { return LittleFS.begin(true); })) return;

    // Start mDNS
    if (!InitComponent("mDNS", []() -> bool { return MDNS.begin(mDNS_NAME); })) return;

    // Start the WebSocket
    InitWebSocket();
    
    // Start the WebServer
    InitWebServer();
}

void loop() {
    if (Serial2.available()) {
        if (start_time == 0) start_time = millis();

        char c = Serial2.read();
        if (c == '\n') {
            Serial.println("Received from Arduino: " + uart_buf);
            uart_buf = "";

            proc_time = millis() - start_time;
            start_time = 0;
            counter += proc_time;
        } else {
            uart_buf += c;
        }
    } else {
        if (counter >= base_delay) {
            Serial2.print("Hello world from ESP32!\n");
            counter = 0;
        } else {
            delay(1);
            counter += 1;
        }
    }
}

// ==========================
//      SETUP FUNCTIONS
// ==========================
template <typename F>
bool InitComponent(const char* componentName, F InitFunction){
    //Serial.print("Starting "); Serial.print(componentName); Serial.print("...");

    int timeoutCount = TIMEOUT;
    while(!InitFunction()){
        delay(1000);
        Serial.print('.');
        timeoutCount--;
        if(timeoutCount == 0){
            Serial.println("\n[ ERR ] COULD NOT START " + String(componentName) + ".");
            return false;
        }
    }

    //Serial.println("\n[ OK ] Started " + String(componentName) + ".");
    return true;
}

void InitWebSocket() {
    ws.onEvent(OnEvent);
    server.addHandler(&ws);
}

void InitWebServer() {
    // Assign the files to serve when requesting an address on the server.
    // currently only "/" is available. Any other address will result in 404: Not found
    server.on("/", HTTP_GET, [](AsyncWebServerRequest* request)
             { request->send(LittleFS, "/webpage/index.html", "text/html"); });

    server.serveStatic("/webpage", LittleFS, "/webpage");
    server.serveStatic("/webserver", LittleFS, "/webserver");
    server.onNotFound(NotFound);

    // Start the server
    server.begin();
}

// =========================================
//  FUNCTIONS CALLED BY WEBSERVER/WEBSOCKET
// =========================================
void OnEvent(AsyncWebSocket* server, AsyncWebSocketClient* client, AwsEventType type, void* arg, uint8_t* payload, size_t len){
    switch(type){
        case WS_EVT_CONNECT:
            Serial.printf("WebSocket client #%u connected from %s\n", client->id(), client->remoteIP().toString().c_str());
            break;
        case WS_EVT_DISCONNECT:
            Serial.printf("WebSocket client #%u disconnected\n", client->id());
            break;
        case WS_EVT_DATA:
            HandleWebSocketMessage(arg, payload, len);
            break;
        default: 
            break;
    }
}

void HandleWebSocketMessage(void* arg, uint8_t* payload, size_t len){
    AwsFrameInfo* info = (AwsFrameInfo*)arg;
    // Make sure that the websocket frame received is actually the one with the message (payload)
    if(info->final && info->index == 0 && info->len == len && info->opcode == WS_TEXT){
        payload[len] = 0;       // null-terminate the payload array so it can be converted to String type
        msg = (char*)payload;

        if(strcmp((char*)payload, "GetValues") == 0){
            //NotifyClients(GetLedStatus());
            Serial.println("Got GetValues");
        }

        Serial.println(msg);
    }
}

void NotFound(AsyncWebServerRequest* request){
    request->send(404, "text/plain", "ERROR 404: PAGE NOT FOUND");
}