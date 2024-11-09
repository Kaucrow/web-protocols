#include <Arduino.h>
#include <AltSoftSerial.h>

AltSoftSerial AltSerial; // RX = 8, TX = 9

String uart_buf;

void setup() {
    // Begin the Serial comm
    Serial.begin(9600);

    while (!Serial) {
        ; // Wait for the Serial comm to be available
    }

    AltSerial.begin(9600);
    
    Serial.println("Setup finished.");
}

void loop() {
    if (AltSerial.available()) {
        char c = AltSerial.read();
        if (c == '\n') {
            Serial.println("Received from ESP32: " + uart_buf);
            AltSerial.print("200 OK\n");
            uart_buf = "";
        } else {
            uart_buf += c;
        }
    }
}