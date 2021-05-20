#include <wiring_private.h>

#define PIN_BLE_SERIAL_X_RX (84ul)
#define PIN_BLE_SERIAL_X_TX (85ul)
#define PAD_BLE_SERIAL_X_RX (SERCOM_RX_PAD_2)
#define PAD_BLE_SERIAL_X_TX (UART_TX_PAD_0)
#define SERCOM_BLE_SERIAL_X sercom0

static Uart rtl_uart(&SERCOM_BLE_SERIAL_X, PIN_BLE_SERIAL_X_RX, PIN_BLE_SERIAL_X_TX, PAD_BLE_SERIAL_X_RX, PAD_BLE_SERIAL_X_TX);

extern "C" {
    void SERCOM0_0_Handler()
    {
        rtl_uart.IrqHandler();
    }
    void SERCOM0_1_Handler()
    {
        rtl_uart.IrqHandler();
    }
    void SERCOM0_2_Handler()
    {
        rtl_uart.IrqHandler();
    }
    void SERCOM0_3_Handler()
    {
        rtl_uart.IrqHandler();
    }
}

void setup()
{
    Serial.begin(115200);
    while(!Serial);
    rtl_uart.begin(614400);

    pinMode(RTL8720D_CHIP_PU, OUTPUT);
    digitalWrite(RTL8720D_CHIP_PU, LOW);
    delay(100);
    digitalWrite(RTL8720D_CHIP_PU, HIGH);
    delay(200);
}

void loop()
{
    if (Serial.available() > 0)
    {
        char c = Serial.read();
        rtl_uart.write(c);
    }
    if (rtl_uart.available() > 0)
    {
        char c = rtl_uart.read();
        Serial.write(c);
    }
}
