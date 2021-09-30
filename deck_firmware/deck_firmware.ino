#include "pb_encode.h"
#include "pb_decode.h"
#include "communique.pb.h"

#include <LiquidCrystal.h>

// Global Constants disgused as program settings
const int NUMBER_OF_BUTTONS = 3;
const unsigned long BUTTON_BOUNCE_DELAY = 500;
const int DISPLAY_COLUMNS = 16;
const int DISPLAY_ROWS = 2;

const int LCD_RS = 12;
const int LCD_EN = 11;
const int LCD_DATA_1 = 5;
const int LCD_DATA_2 = 4;
const int LCD_DATA_3 = 3;
const int LCD_DATA_4 = 2;
const int LCD_BRIGHTNESS = 10;
const int LCD_CONTRAST = 9;
const int LCD_FADETIME = 35; // 0 for instant. Blocking.

// handled by protobuf
bool MESSAGE_RECEIVED = false; // currently processing message
int LCD_TARGET = 0; // LCD screen illumination goal
long DELAY_TIME = millis();
int CURRENT_BRIGHTNESS_STORAGE = 0; // analogRead may give inconsistent results

LiquidCrystal lcd(LCD_RS, LCD_EN, LCD_DATA_1, LCD_DATA_2, LCD_DATA_3, LCD_DATA_4);

class BadVec
{
  private:
    char _storage[DISPLAY_ROWS][DISPLAY_COLUMNS];
    int _elements = -1;
  
  public:
    void push_back(char *v, int passed_in_length)
    {
        _elements++;
        for (int i = 0; i < DISPLAY_COLUMNS; i++)
        {
            _storage[_elements][i] = v[i];
        }
        if (passed_in_length < DISPLAY_COLUMNS) {
          for (int i = passed_in_length; i < DISPLAY_COLUMNS; i++) {
            _storage[_elements][i] = " ";
          }
        }
    }
    char* get(int index) {
      if (index >= 0) {
        return _storage[index];
      }
    }
    int length() {
      return _elements+1; // avoid off-by-one
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
    LCD_Init();
    Serial.begin(115200);
    while(!Serial); // wait for serial connection
    lcd.print("Connected!      ");
    lcd.setCursor(0,1);
    lcd.print("Awaiting command");
    LCD_fadeout();
}

void LCD_Init() {
    // initialize screen
    lcd.begin(DISPLAY_COLUMNS, DISPLAY_ROWS);
    pinMode(LCD_CONTRAST, OUTPUT);
    pinMode(LCD_BRIGHTNESS, OUTPUT);
    digitalWrite(LCD_CONTRAST, LOW);
    lcd.print("Initializing");
    lcd.setCursor(0,1);
    lcd.print("serial conn");  
    lcd.setCursor(0,0);
    LCD_fadein();
}

void LCD_setMessage(char* line1, char* line2) {
  LCD_clear();
  lcd.setCursor(0,0);
  lcd.print(line1);
  lcd.setCursor(0,1);
  lcd.print(line2);
}

void LCD_fadeout() {
  for (int l = 51; l > -1; l--) {
    analogWrite(LCD_BRIGHTNESS, l * 5);
    CURRENT_BRIGHTNESS_STORAGE = l * 5;
    delay(LCD_FADETIME);
  }
}

void LCD_fadein() {
  for (int l = 0; l < 51; l++) {
    analogWrite(LCD_BRIGHTNESS, l * 5);
    CURRENT_BRIGHTNESS_STORAGE = l * 5;
    delay(LCD_FADETIME);
  }  
}

void LCD_clear() {
  char clearStr[DISPLAY_COLUMNS];
  for (int i = 0; i < DISPLAY_COLUMNS; i++) {
    clearStr[i] = " ";
  }
  for (int i = 0; i < DISPLAY_ROWS; i++) {
    lcd.setCursor(0,i);
    lcd.print(clearStr);
  }
  lcd.setCursor(0,0);
}

/* LCD_manage()
 * It's messy, for sure, but doing it this way ensures that it's at least
 * less blocking than calling LCD_fadein and LCD_fadeout. 
 * 
 * Uses a storage value, CURRENT_BRIGHTNESS_STORAGE, because using analogRead
 * will give inconsistent results that may end up causing the display to stay
 * on. This way we don't get jitter and can stay within bounds.
 */
void LCD_manage() {
  if (MESSAGE_RECEIVED) {
    int current_brightness = CURRENT_BRIGHTNESS_STORAGE;
    // fade in
    if (current_brightness < LCD_TARGET) {
      // surpassing target
      if (current_brightness + 5 > LCD_TARGET) {
        current_brightness = LCD_TARGET - 5;
      }
      analogWrite(LCD_BRIGHTNESS, current_brightness + 5);
      current_brightness += 5;
    }
    // fade out
    if (current_brightness > LCD_TARGET) {
      // underflow
      if (current_brightness - 5 > current_brightness) {
        current_brightness = 5;
      }
      // surpassing target
      if (current_brightness - 5 < LCD_TARGET) {
        current_brightness = LCD_TARGET + 5;
      }
      analogWrite(LCD_BRIGHTNESS, current_brightness - 5);
      current_brightness -= 5;
    }
    // if target is reached
    if (current_brightness == LCD_TARGET) {
      // and the delay time has passed
      if (millis() - DELAY_TIME > 0) {
        // start winding down
        LCD_TARGET = 0;
      }
    }
    // delay fade effect if necessary
    if (current_brightness == LCD_TARGET && millis() - DELAY_TIME > 0) {
      MESSAGE_RECEIVED = false;
    } else {
      delay(LCD_FADETIME);
    }
    CURRENT_BRIGHTNESS_STORAGE = current_brightness;
  }
}

// Called every tick
void loop() {    
  LCD_manage();
  
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
 
      // Display it on the LCD
      if (lines.length() > 0) {
        LCD_clear();
        for (int i = 0; i < lines.length(); i++) {
          lcd.setCursor(0,i);
          lcd.print((char*)lines.get(i));
        }
        LCD_TARGET = msg.brightness;
        MESSAGE_RECEIVED = true;
        DELAY_TIME = millis() + msg.duration_ms;
      }
    }
  }
}
