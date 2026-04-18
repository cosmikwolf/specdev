# Firmware Patterns to Detect in C Source (Reverse-Engineering)

This file defines what C language patterns to look for when reverse-engineering firmware source code into spec YAML. Every rule couples a firmware domain concept with the C constructs that indicate it.

---

## ISR Involvement → Handler Functions and Interrupt Attributes

Look for these patterns to detect ISR-related code and set `concurrency.isr_shared: true`:

- Functions ending in `_IRQHandler` → ARM Cortex-M NVIC interrupt handler (e.g., `USART1_IRQHandler`, `TIM2_IRQHandler`)
- Functions ending in `_isr` or `_Handler` → generic ISR naming convention
- `IRAM_ATTR` attribute → ESP32 ISR function placed in IRAM: `void IRAM_ATTR gpio_isr_handler(void *arg)`
- `__attribute__((interrupt))` → GCC interrupt attribute for ISR functions
- `__attribute__((isr))` → alternative ISR attribute
- `ISR()` macro → AVR interrupt service routine
- `void EXTI0_IRQHandler(void)` → STM32 EXTI interrupt handler pattern
- `esp_intr_alloc()` or `gpio_isr_handler_add()` → ESP-IDF interrupt registration
- `NVIC_EnableIRQ()`, `NVIC_SetPriority()` → ARM Cortex-M interrupt configuration
- Any function registered via interrupt vector table assignment

## Volatile Usage → ISR-Shared Variable Detection

Presence of `volatile` qualifier indicates ISR-shared data:

- `static volatile uint32_t counter;` → ISR-shared counter variable
- `volatile bool flag;` → ISR-shared boolean flag
- `volatile uint8_t buffer[N];` → ISR-shared buffer (likely DMA or ring buffer)
- `volatile struct { ... } shared_data;` → ISR-shared compound type
- Map each `volatile` variable to an entry in `concurrency.isr_shared_vars`
- `volatile` on a pointer: `volatile uint8_t *ptr` (data is volatile) vs `uint8_t * volatile ptr` (pointer itself is volatile) — both indicate ISR involvement
- `sig_atomic_t` usage → signal/ISR-safe atomic variable

## IRQ Protection → Critical Section Patterns

Detect ISR protection strategy for `concurrency.protection`:

**Bare-metal ARM Cortex-M (`protection: disable_irq`):**
- `__disable_irq()` / `__enable_irq()` — CMSIS intrinsics
- `__set_PRIMASK(1)` / `__set_PRIMASK(0)` — direct PRIMASK manipulation
- `__set_BASEPRI(level)` — priority-based masking

**FreeRTOS (`protection: disable_irq` or `protection: mutex`):**
- `taskENTER_CRITICAL()` / `taskEXIT_CRITICAL()` — FreeRTOS critical section (disables scheduler)
- `portENTER_CRITICAL()` / `portEXIT_CRITICAL()` — port-level critical section
- `portDISABLE_INTERRUPTS()` / `portENABLE_INTERRUPTS()` — interrupt disable
- `taskENTER_CRITICAL_FROM_ISR()` / `taskEXIT_CRITICAL_FROM_ISR()` — ISR-safe critical section

**ESP-IDF specific:**
- `portMUX_TYPE mux = portMUX_INITIALIZER_UNLOCKED;` + `taskENTER_CRITICAL(&mux)` → ESP32 spinlock
- `esp_intr_disable()` / `esp_intr_enable()` → per-interrupt disable

**AVR:**
- `cli()` / `sei()` — clear/set global interrupt flag
- `ATOMIC_BLOCK(ATOMIC_RESTORESTATE) { ... }` — avr-libc atomic wrapper

## Mutex Patterns → Thread Safety Detection

Mutex usage indicates `concurrency.thread_safe: true` and `protection: mutex`:

**FreeRTOS:**
- `SemaphoreHandle_t mutex;` → mutex handle declaration
- `xSemaphoreCreateMutex()` → mutex creation in init
- `xSemaphoreTake(mutex, portMAX_DELAY)` / `xSemaphoreGive(mutex)` → lock/unlock
- `xSemaphoreTakeFromISR()` / `xSemaphoreGiveFromISR()` → ISR-safe semaphore (not mutex, but indicates ISR interaction)
- `xSemaphoreCreateBinary()` → binary semaphore (ISR-to-task signaling)

**CMSIS-RTOS v2:**
- `osMutexNew()` → mutex creation
- `osMutexAcquire()` / `osMutexRelease()` → lock/unlock

**POSIX (Linux-targeted firmware):**
- `pthread_mutex_init()` → mutex creation
- `pthread_mutex_lock()` / `pthread_mutex_unlock()` → lock/unlock

## Static Allocation → Absence of Dynamic Memory

Determine `memory.allocation` from heap usage signals:

- Absence of `malloc`, `calloc`, `realloc`, `free` in module source → `allocation: static` (HIGH confidence)
- Absence of `strdup`, `strndup`, `asprintf` → reinforces static allocation
- Presence of `static struct` instances at file scope → static allocation pattern
- Fixed-size arrays: `uint8_t buffer[256]` → static buffers
- FreeRTOS: `pvPortMalloc()` / `vPortFree()` → `allocation: heap`
- ESP-IDF: `heap_caps_malloc()` → `allocation: heap`
- Any `malloc` / `calloc` / `realloc` / `free` → `allocation: heap`
- Check for `#define` size constants near buffers → extract for `memory` spec fields

## Error Handling → Return Code and Enum Patterns

Detect `error_handling` style from function signatures and error types:

**return_code (most common in C firmware):**
- `typedef enum { MODULE_OK = 0, MODULE_ERR_... = -1, ... } module_error_t;` → error enum with OK=0
- Functions returning `int` where 0 is success → return code pattern
- `esp_err_t` return type → ESP-IDF error code convention
- `HAL_StatusTypeDef` → STM32 HAL error convention

**callback:**
- `typedef void (*error_cb_t)(int err_code, void *ctx);` → error callback type
- `module_register_error_handler(...)` → callback registration function

**output_pointer:**
- `int module_read(..., uint8_t *out_data, size_t *out_len);` → output via pointer with int return
- Parameters prefixed with `out_` or `p_` → output pointer convention

**errno_style:**
- `errno` usage or `perror()` calls → POSIX errno pattern (rare in embedded)

Extract error variant names from enum: `MODULE_ERR_TIMEOUT`, `MODULE_ERR_INVALID_PARAM`, etc.

## RTOS Detection → Framework-Specific Includes and API Calls

Identify RTOS/framework for `architecture.yaml`:

**FreeRTOS:**
- `#include "FreeRTOS.h"`, `#include "task.h"`, `#include "queue.h"`, `#include "semphr.h"`
- `xTaskCreate()`, `vTaskDelay()`, `xQueueSend()`, `xQueueReceive()`
- `configTICK_RATE_HZ`, `pdMS_TO_TICKS()` → FreeRTOS timing

**Zephyr:**
- `#include <zephyr/kernel.h>`, `#include <zephyr/device.h>`
- `k_thread_create()`, `k_sem_take()`, `k_msgq_put()`

**CMSIS-RTOS v2:**
- `#include "cmsis_os2.h"`
- `osThreadNew()`, `osMutexNew()`, `osDelay()`

**ThreadX:**
- `#include "tx_api.h"`
- `tx_thread_create()`, `tx_semaphore_get()`

**No RTOS (bare-metal):**
- `while(1) { }` main loop without task creation
- SysTick or timer-based scheduling without RTOS primitives
- Absence of any RTOS include → `rtos: none`

## MCU Detection → Include Patterns and SDK Indicators

Identify target MCU from source includes:

- `#include "stm32f4xx.h"` or `#include "stm32f4xx_hal.h"` → STM32F4 series
- `#include "stm32h7xx.h"` → STM32H7 series
- `#include "esp_system.h"`, `#include "esp_log.h"`, `#include "sdkconfig.h"` → ESP32 (check `CONFIG_IDF_TARGET_*` for variant)
- `#include "nrf52840.h"` or `#include "nrf_gpio.h"` → nRF52840
- `#include "pico/stdlib.h"` → RP2040
- `#include <avr/io.h>` → AVR (ATmega/ATtiny)
- `#include "fsl_common.h"` → NXP i.MX/Kinetis
- Linker script memory regions: `FLASH (rx)`, `RAM (rwx)`, `IRAM` → confirms MCU family

## Module Discovery → Header and Source File Pairs

Identify module boundaries:

- Each `.h` file in `src/include/` → potential module public API
- Each subdirectory in `src/` with `.c` files → module implementation
- `#include "module_name.h"` in source → module is a dependency
- `typedef struct module_s module_t;` → opaque type pattern → confirmed module
- Functions prefixed with `module_name_` → C namespace convention for module scoping
- `module_init()` / `module_deinit()` function pair → module lifecycle

## Dependency Detection → #include Analysis

Map module dependencies from include statements:

- `#include "other_module.h"` in module source → `depends_on: [other_module]` (HIGH confidence)
- `#include "driver/gpio.h"` → external dep on ESP-IDF GPIO driver
- `#include "hal_spi.h"` → dependency on SPI HAL module
- Includes in the `.c` file (not the `.h`) → implementation dependency (still record in `depends_on`)
- Circular includes → flag as dependency violation
- Conditional includes `#ifdef FEATURE_X` `#include "optional.h"` `#endif` → conditional dependency, note in spec

## Peripheral Ownership → Init Calls and Register Access

Detect `owns` resources:

- `gpio_config_t io_conf = { .pin_bit_mask = (1 << PIN), ... }; gpio_config(&io_conf);` → owns GPIO pin
- `spi_bus_initialize(SPI2_HOST, ...)` → owns SPI bus
- `i2c_driver_install(I2C_NUM_0, ...)` → owns I2C bus
- `uart_driver_install(UART_NUM_1, ...)` → owns UART peripheral
- `GPIOA->MODER |= ...` → direct register access → owns GPIOA peripheral
- `SPI2->CR1 = ...` → direct register access → owns SPI2 peripheral
- `#define LED_PIN GPIO_NUM_2` → pin assignment constant → owns that pin
- Timer configuration: `TIM2->ARR = ...` or `timer_init(TIMER_GROUP_0, ...)` → owns timer

## Layer Inference → Abstraction Level from Code Patterns

Determine module `layer`:

- Direct register access: `PERIPHERAL->REGISTER = value;` → `layer: hal`
- Raw register definitions: `#define REG_BASE 0x40000000` → `layer: pac`
- Wrapping HAL calls: calls `HAL_GPIO_WritePin()` or `gpio_set_level()`, provides higher-level API → `layer: driver`
- Protocol implementation: Bluetooth stack, WiFi management, USB class → `layer: middleware`
- `main()` function, task creation, module init sequence → `layer: application`
- Helper utilities: CRC, checksum, ring buffer → `layer: middleware` or `driver` depending on hardware involvement

## Display Detection → Graphics Library Function Calls

Identify display hardware for UI spec:

- `u8g2_DrawStr()`, `u8g2_DrawBox()`, `u8g2_SendBuffer()` → u8g2 display library
- `ssd1306_*` function family → SSD1306 OLED driver
- `lv_label_create()`, `lv_obj_set_style_*()` → LVGL graphics library
- `GFX_DrawString()`, `GFX_FillRect()` → custom or vendor graphics library
- Display init parameters reveal resolution: `u8g2_Setup_ssd1306_128x64_noname_f(...)` → 128x64

## Input Detection → GPIO Read and Button State Machines

Identify input devices:

- `gpio_get_level(PIN)` → GPIO input read (button or switch)
- `adc1_get_raw(CHANNEL)` → ADC read (slider, potentiometer)
- Debounce patterns: `if (now - last_press > DEBOUNCE_MS)` → button with debounce
- State variables: `button_state`, `pressed`, `released`, `held` → button state machine
- Encoder patterns: quadrature decoding, `encoder_count`, `direction` → rotary encoder
- Long press detection: `if (press_duration > LONG_PRESS_MS)` → long press gesture

## LED Detection → Color Setting and Pattern Functions

Identify LED indicators:

- `led_strip_set_pixel(strip, index, r, g, b)` → addressable RGB LED strip (ESP-IDF)
- `ws2812_set_color(index, color)` → WS2812/NeoPixel LED
- `gpio_set_level(LED_PIN, 1)` / `gpio_set_level(LED_PIN, 0)` → single-color LED on/off
- `set_rgb(r, g, b)` or `set_color(color)` → RGB LED control
- Blink patterns: `led_blink(period_ms)`, timer-based toggle → LED animation
- Conditional color: `if (status == CONNECTED) set_color(GREEN)` → status-to-LED binding

## FSM Detection → State Enum and Switch-on-State Patterns

Identify state machines:

- `typedef enum { STATE_IDLE, STATE_CONNECTING, ... }` → FSM state enum
- `enum` with `STATE_` or `state_` prefix → state definitions
- `typedef enum { EVENT_CONNECT, EVENT_TIMEOUT, ... }` → FSM event enum
- `switch (state) { case STATE_IDLE: ... }` → switch-based FSM dispatch
- `state = STATE_CONNECTED;` → state transition assignment
- 2D arrays: `next_state[current][event]` → table-driven FSM
- Functions named `fsm_dispatch`, `handle_event`, `process_event` → FSM entry point

## Blocking Detection → Function Names and Wait Patterns

Determine `blocking` attribute for interface functions:

- Function names containing `_blocking`, `_wait`, `_poll`, `_sync` → `blocking: true`
- `while (!(STATUS_REG & READY_BIT)) { }` → busy-wait loop → `blocking: true`
- `vTaskDelay()`, `osDelay()`, `usleep()` → blocking delay inside function → `blocking: true`
- Timeout parameters: `uint32_t timeout_ms` → may block up to timeout → `blocking: true`
- Callback-based: function returns immediately, result via callback → `blocking: false`
- DMA transfer initiation without wait → `blocking: false`
- Functions that only set registers and return → `blocking: false`
