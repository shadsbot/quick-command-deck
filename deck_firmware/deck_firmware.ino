#include "pb_encode.h"
#include "pb_decode.h"
#include "communique.pb.h"

// Global Constants disgused as program settings
const int NUMBER_OF_BUTTONS = 3;
const unsigned long BUTTON_BOUNCE_DELAY = 500;

class KeyButton
{
  private:
    unsigned int _pin;
    unsigned long _last_pressed;

  public:
    unsigned int pin() { return _pin; }
    bool pressed() { 
      if (digitalRead(_pin) == 0) {
        _last_pressed = millis();
        return true;
      }
      return false;
    }
    bool valid_press() { return (millis() - _last_pressed > BUTTON_BOUNCE_DELAY); }
    KeyButton(int);
};

KeyButton::KeyButton(int a) {
  _pin = a;
  pinMode(a, INPUT_PULLUP);
  _last_pressed = millis();
}

KeyButton buttons[] = { KeyButton(6), KeyButton(7), KeyButton(8) };


// Called once on startup
void setup() {
    Serial.begin(115200);
    while(!Serial); // wait for serial connection
}

// Called every tick
void loop() {
  for (int i = 0; i < NUMBER_OF_BUTTONS; i++) {
    if (buttons[i].valid_press()) {
      if (buttons[i].pressed()) {
        char buffer[256];
        ButtonPushed bp_message = ButtonPushed{i};
        pb_ostream_t stream = pb_ostream_from_buffer(buffer, sizeof(buffer));
        pb_encode(&stream, ButtonPushed_fields, &bp_message);
        Serial.write(buffer, stream.bytes_written);
      }
    }
  }
}
