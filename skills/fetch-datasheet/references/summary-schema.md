# `summary.yaml` schema for /fetch-datasheet

The `summary.yaml` emitted by `/fetch-datasheet` Step 7 has the shape below. Every field is optional unless marked **required**; write `null` for any field not clearly stated in the parsed datasheet text.

## Top-level identity

- `part` (**required**) — canonical manufacturer part number (e.g. `"ESP32-S3-WROOM-1"`).
- `manufacturer` (**required**) — e.g. `"Espressif"`.
- `document_type` (**required**) — one of `"datasheet"`, `"reference_manual"`, `"technical_reference"`, `"user_guide"`, `"errata"`.
- `source_url` — origin URL of the PDF.
- `source_file` — relative path of the downloaded PDF under `docs/datasheets/<part>/<doc_type>/`.

## Overview

```yaml
overview:
  description: string
  core: string            # e.g. "dual-core Xtensa LX7"
  max_clock_mhz: number
  flash: string           # e.g. "8 MB"
  sram: string            # e.g. "512 KB"
```

## Peripherals

```yaml
peripherals:
  - name: string          # e.g. "SPI"
    count: integer        # number of instances
    notes: string
```

## Pins

```yaml
pins:
  total_gpio: integer
  adc_pins: integer
  dac_pins: integer
  touch_pins: integer
  strapping_pins: integer
```

## Electrical

```yaml
electrical:
  vdd_range: string       # e.g. "3.0 V – 3.6 V"
  io_voltage: string
  max_gpio_current_ma: number
  deep_sleep_current_ua: number
```

## Timing

```yaml
timing:
  startup_time_ms: number
  clock_sources:
    - name: string        # e.g. "XTAL", "PLL"
      frequency_mhz: number
```

## Register maps

```yaml
register_maps:
  - peripheral: string
    chapter_file: string  # relative path under parsed/
    base_address: string  # hex, e.g. "0x3FF4_0000"
```

## Interfaces

```yaml
interfaces:
  - name: string          # e.g. "USB 1.1 OTG"
    max_clock_mhz: number
    modes: [string, ...]  # e.g. ["host", "device"]
```

## Power modes

```yaml
power_modes:
  - name: string          # e.g. "deep sleep"
    typical_current_ma: number
```

## Notes

- `notes` — free-form string.

## Authoring rules

- **Never invent values.** Write `null` for any field not clearly stated in the parsed text. Downstream skills trust this file to constrain spec authoring — hallucinations cascade.
- Units are part of the field name where practical (`_ma`, `_mhz`, `_ms`, `_us`). String-valued fields preserve the datasheet's original unit string when conversion would be lossy.
- Lists may be empty (`peripherals: []`) when absent; do not omit the key entirely.
