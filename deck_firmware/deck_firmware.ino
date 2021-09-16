
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
    while(!Serial);
}

// Called every tick
void loop() {
  for (int i = 0; i < NUMBER_OF_BUTTONS; i++) {
    if (buttons[i].valid_press()) {
      if (buttons[i].pressed()) {
        char buffer[256];
        sprintf(buffer, "Button on pin %d was pressed", buttons[i].pin());
        Serial.print(buffer);
      }
    }
  }
}

bool same(int a[], int b[]) {
  for (int i = 0; i < sizeof a / *a; i++) {
    if (a[i] != b[i]) {
      return false;
    }
  }
  return true;
}
