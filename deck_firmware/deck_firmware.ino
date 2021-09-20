#include "pb_encode.h"
#include "pb_decode.h"
#include "communique.pb.h"

// Global Constants disgused as program settings
const int NUMBER_OF_BUTTONS = 3;
const unsigned long BUTTON_BOUNCE_DELAY = 500;
const int DISPLAY_COLUMNS = 16;
const int DISPLAY_ROWS = 2;

class BadVec
{
  private:
    char* _storage[DISPLAY_ROWS][DISPLAY_COLUMNS];
    int _elements = -1;
  
  public:
    void push_back(char* v, int passed_in_length) {
      _elements++;
      for (int i = 0; i < DISPLAY_COLUMNS; i++) {
        if (i < passed_in_length) {
          _storage[_elements][i] = v[i];
        } else {
          // clear garbage data
          _storage[_elements][i] = ' ';
        }
      }
    }
    char** get(int index) {
      if (index >= 0) {
        return _storage[index];
      }
    }
    int length() {
      return _elements;
    }
};

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

// Callback handling for how nanopb handles strings
bool read_string(pb_istream_t *stream, const pb_field_iter_t *field, void **arg) {
  while (stream->bytes_left) {
    char* line[DISPLAY_COLUMNS] = {""};
    int stream_size = stream->bytes_left;
    // Should dump whatever's in stream into a buffer
    pb_read(stream, *line, stream->bytes_left);
    // Dump that buffer into an array
    static_cast<BadVec*>(*arg)->push_back(*line, stream_size);
  }
  return true;
}

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
  
  if (Serial.available() > 0) {
    char buffer[256];
    int returned_bytes = Serial.readBytes(buffer, 256);
    if (returned_bytes > 0) {
      // Attempt to decode message coming from software
      // Setting up
      DisplayText msg = DisplayText_init_zero;
      pb_istream_t stream = pb_istream_from_buffer(buffer, sizeof(buffer));

      // Display's lines to show
      BadVec lines;
      msg.line.funcs.decode = &read_string;
      msg.line.arg = &lines;

      // And the actual decode attempt
      auto status = pb_decode(&stream, DisplayText_fields, &msg);
    }
  }
}
