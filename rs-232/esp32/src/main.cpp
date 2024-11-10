#include <Arduino.h>
#include <Wifi.h>
#include <LittleFS.h>
#include <ArduinoJson.h>
#include <ESPAsyncWebServer.h>
#include <ESPmDNS.h>

#define RXD2 16
#define TXD2 17
#define TIMEOUT 5       // Components startup timeout

const String AP_NAME = "ESP32_AP";
const String mDNS_NAME = "ESP32";

AsyncWebServer server(80);
AsyncWebSocket ws("/ws");

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
void SendMessageFrame(JsonDocument& rec_json);
void SendCreateFileFrame(JsonDocument& rec_json);
void SendDeleteFileFrame(JsonDocument& rec_json);
void SendCopyFileFrame(JsonDocument& rec_json);

void setup() {
    Serial.begin(9600);
    Serial2.begin(9600, SERIAL_8N1, RXD2, TXD2);

    while (!Serial) {
        ;   // Wait for the Serial port to be available
    }
    
    WiFi.softAP(AP_NAME);

    Serial.print("\nWebServer listening on: ");
    Serial.println(WiFi.softAPIP());
    
    // Start LittleFS
    if (!InitComponent("LittleFS", []() -> bool { return LittleFS.begin(true); })) return;

    // Start mDNS
    if (!InitComponent("mDNS", []() -> bool { return MDNS.begin(mDNS_NAME); })) return;

    // Start the WebSocket server
    InitWebSocket();
    
    // Start the WebServer
    InitWebServer();
}

void loop() {
    if (Serial2.available()) {
        char c = Serial2.read();

        // Ignore NULL and other control characters (e.g., '\0')
        if (c == 0 || c == 127) {
            return;
        }

        if (c == '\n') {
            if (uart_buf == "OK") {
                Serial.println("Received UART OK");
            } else {
                Serial.println("Received from UART: " + uart_buf);
            }
            uart_buf = "";
        } else {
            uart_buf += c;
        }
    }

    ws.cleanupClients();
}

// ==========================
//      SETUP FUNCTIONS
// ==========================
template <typename F>
bool InitComponent(const char* componentName, F InitFunction){
    Serial.print("-> Starting "); Serial.print(componentName); Serial.println("...");

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

    Serial.println("[ OK ] Started " + String(componentName) + ".");

    return true;
}

void InitWebSocket() {
    ws.onEvent(OnEvent);
    server.addHandler(&ws);
}

void InitWebServer() {
    // Assign the files to serve when requesting an address on the server.
    // Only "/" is available. Any other address will result in 404: Not found
    server.on("/", HTTP_GET, [](AsyncWebServerRequest* request)
             { request->send(LittleFS, "/webpage/index.html", "text/html"); });

    server.serveStatic("/webpage", LittleFS, "/webpage");
    server.serveStatic("/websockets", LittleFS, "/websockets");
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
        String msg = (char*)payload;

        if(strcmp((char*)payload, "GetValues") == 0){
            // Logic for updating the connected WebSocket clients
            Serial.println("Got GetValues");
        }

        JsonDocument rec_json;
        DeserializationError err = deserializeJson(rec_json, msg);

        if (err) {
            Serial.print("Failed to parse JSON: ");
            Serial.println(err.c_str());
            return;
        }

        Serial.println("Received from WebSocket: " + msg);

        if (rec_json["message"].is<String>()) {
            SendMessageFrame(rec_json);
        } else if (rec_json["content"].is<String>()) {
            SendCreateFileFrame(rec_json);
        } else if (rec_json["path"].is<String>()) {
            SendDeleteFileFrame(rec_json);
        } else if (rec_json["fromPath"].is<String>()) {
            SendCopyFileFrame(rec_json);
        }
    }
}

void NotFound(AsyncWebServerRequest* request){
    request->send(404, "text/plain", "ERROR 404: PAGE NOT FOUND");
}

void SendMessageFrame(JsonDocument& rec_json) {
    String msg = rec_json["message"];
    String frame = "init^send^" + msg + "^endData^close";
    Serial2.print(frame + '\n');
}

void SendCreateFileFrame(JsonDocument& rec_json) {
    String path = rec_json["path"];
    String content = rec_json["content"];
    String frame = "init^create^" + content + ">" + path + "^endData^close";
    Serial2.print(frame + '\n');
}

void SendDeleteFileFrame(JsonDocument& rec_json) {
    String path = rec_json["path"];
    String frame = "init^delete^" + path + "^endData^close";
    Serial2.print(frame + '\n');
}

void SendCopyFileFrame(JsonDocument& rec_json) {
    String from_path = rec_json["fromPath"];
    String to_path = rec_json["toPath"];
    String frame = "init^copy^" + from_path + ">" + to_path + "^endData^close";
    Serial2.print(frame + '\n');
}