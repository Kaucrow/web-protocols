#include "Arduino.h"

#define RXD2 16
#define TXD2 17

String uart_buf;

unsigned long base_delay = 1000;
unsigned long counter = 0;
unsigned long start_time = 0;
unsigned long proc_time = 0;

void setup() {
    Serial.begin(9600);
    Serial2.begin(9600, SERIAL_8N1, RXD2, TXD2);
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