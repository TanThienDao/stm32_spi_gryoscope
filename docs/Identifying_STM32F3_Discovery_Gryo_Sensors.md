# STM32F3DISCOVERY Onboard Gyroscope Reference

Depending on the physical board revision of your **STM32F3DISCOVERY**, the onboard SPI gyroscope uses one of two parts:

* **L3GD20 / L3GD20H** — Board revisions up to **Rev D01**
* **I3G4250D** — Board revisions starting from **Rev E02**

Because both sensors belong to the same STMicroelectronics MEMS family, they share the exact same register map and SPI protocol parameters (`SPI Mode 3`, `0x0F` address for `WHO_AM_I`). However, their register identification values differ.

---

### Gyroscope `WHO_AM_I` (`0x0F`) Register Mapping

| Sensor Part | Board Revision | Expected `WHO_AM_I` Byte |
| :--- | :--- |:------------------------:|
| **I3G4250D** | Rev E02 or newer |    `0xD3 (11010011)`     |
| **L3GD20** | Up to Rev D01 |    `0xD4 (11010100)`     |
| **L3GD20H** | Up to Rev D01 (varies) |    `0xD7 (11010111)`     |

---

### Hardware Configuration Summary

* **Protocol:** SPI (Full-Duplex)
* **SPI Mode:** Mode 3 (`CPOL` = 1, `CPHA` = 1)
* **Bus Line (SPI1):** `PA5` (SCK), `PA6` (MISO), `PA7` (MOSI)
* **Chip Select Pin:** `PE3` (Active Low)

***
# Data Sheet References:
## I3G4250D
![l3gd20h_p34-1.jpg](docs/Screenshots/l3gd20h_p34-1.jpg)
![l3gd20h_p36-1.jpg](docs/Screenshots/l3gd20h_p36-1.jpg)
## L3GD20
![DS_l3gd20_p29-1.jpg](docs/Screenshots/DS_l3gd20_p29-1.jpg)
![DS_l3gd20_p31-1.jpg](docs/Screenshots/DS_l3gd20_p31-1.jpg)
## L3GD20H
![l3gd20h_p34-1.jpg](docs/Screenshots/l3gd20h_p34-1.jpg)
![l3gd20h_p36-1.jpg](docs/Screenshots/l3gd20h_p36-1.jpg)