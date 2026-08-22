# SPI Clock Modes & Configuration Guide

An **SPI mode** determines the timing rules for transmitting and sampling data bits on an SPI bus. The host controller configures the bus using two key parameters: **CPOL** (Clock Polarity) and **CPHA** (Clock Phase)[cite: 1].

---

### Fundamental Concepts

* **Clock Polarity (CPOL):** Defines the state of the Serial Clock line (`SCK`) when idle[cite: 1].
    * `CPOL = 0`: Clock idles **LOW** (`0V`)[cite: 1].
    * `CPOL = 1`: Clock idles **HIGH** (`3.3V` / `5V`)[cite: 1].

* **Clock Phase (CPHA):** Determines which clock transition triggers data sampling[cite: 1].
    * `CPHA = 0`: Data is sampled on the **1st clock edge** (transition out of the idle state)[cite: 1].
    * `CPHA = 1`: Data is sampled on the **2nd clock edge** (transition back into the idle state)[cite: 1].

---

### The 4 Standard SPI Modes

Combining `CPOL` and `CPHA` yields four operating modes[cite: 1]:

| SPI Mode | CPOL | CPHA | Idle Clock State | Data Sampled On |
| :--- | :---: | :---: | :--- | :--- |
| **Mode 0** | `0` | `0` | Low (`0V`) | 1st transition (Rising Edge)[cite: 1] |
| **Mode 1** | `0` | `1` | Low (`0V`) | 2nd transition (Falling Edge)[cite: 1] |
| **Mode 2** | `1` | `0` | High (`3.3V`) | 1st transition (Falling Edge)[cite: 1] |
| **Mode 3** | `1` | `1` | High (`3.3V`) | 2nd transition (Rising Edge)[cite: 1] |

---

### Application: I3G4250D / L3GD20 Gyroscope

The onboard gyroscope requires **SPI Mode 3** (or Mode 0 on compatible hardware)[cite: 1]:

* **CPOL = 1:** The `SCK` line stays **HIGH** when Chip Select (`CS`) is idle[cite: 1].
* **CPHA = 1:** Data transitions on `MOSI`/`MISO` occur on the falling clock edge and are captured on the **rising clock edge**[cite: 1].

> **Warning:** Mismatched modes cause sampling timing errors, resulting in corrupted bus readings like invalid `WHO_AM_I` responses (`0x00` or `0xFF`)[cite: 1].