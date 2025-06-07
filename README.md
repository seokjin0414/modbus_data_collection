# High-Performance Parallel IoT Sensor Data Collection Client/Server Example with Axum

This project demonstrates a high-performance, parallel IoT sensor data collection client-server system built with **Rust** using the [Axum](https://github.com/tokio-rs/axum) web framework.

The current implementation includes data collection from **GEMS_3005** sensors using Modbus communication.  
Planned features include support for additional protocols such as raw socket communication and **SIHAS**.

---

## Features

- Each collection point can act as a **collector server on a client PC**.
- Sensor **memory map information** and **collection point configuration** are cached locally (L1 cache) at server startup to significantly improve data collection speed.
  - *(Note: "L1 cache" typically refers to CPU cache, but here it means an application-level local memory cache.)*
- All connections to sensors and data collection processes are fully **asynchronous and parallelized**, ensuring optimal collection speed regardless of the number of sensors.
- For example, the implemented **GEMS_3005** sensor collects data from 18 different addresses (addr) at each collection point in parallel, and the logic is optimized for this use case.

---

## Environment & Dependencies

- Latest stable version of **Rust**
- [Axum](https://github.com/tokio-rs/axum)
- For all other libraries and dependencies, please refer to the **Cargo.toml** file.

---

## Note

- Currently, legacy and newly implemented code coexist in this project. Refactoring and clean-up are planned.

---

## Contact

If you have any questions, need additional support, or would like to collaborate, please feel free to reach out:

- **Email:** sars21@hanmail.net  
- **LinkedIn:** [https://www.linkedin.com/in/seokjin-shin/](https://www.linkedin.com/in/seokjin-shin/)

Feel free to get in touch anytime!
