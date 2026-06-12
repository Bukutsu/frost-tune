# Frost-Tune

# ⚠️ DEPRECATED — MOVED TO GLACIER EQ

> [!WARNING]
> **THIS PROJECT IS DEPRECATED AND NO LONGER MAINTAINED.**
> It has been completely rebuilt from scratch as **[Glacier EQ](https://github.com/Bukutsu/glacier-eq)**.
> Please visit the new repository for the latest updates, downloads, and support:
> 👉 **[https://github.com/Bukutsu/glacier-eq](https://github.com/Bukutsu/glacier-eq)**

---

Frost-Tune was a native, offline, zero-latency parametric EQ editor for USB DACs built with Rust, Iced, and Tokio.

Glacier EQ is the successor — a cross-platform parametric EQ editor with:
- Full-featured web UI (React + TypeScript)
- Desktop (Linux, Windows, macOS) and Android support via Tauri
- Direct USB HID communication with compatible DACs
- 10-band parametric EQ with real-time visualization
- AutoEQ optimization engine
- Profile management and import/export

👉 **[Visit Glacier EQ on GitHub](https://github.com/Bukutsu/glacier-eq)**

## Migration

If you have existing Frost-Tune EQ profiles, you can manually recreate them in Glacier EQ's profile system. The underlying HID protocol for supported DACs is the same.

## License

MIT. See `LICENSE`.

## Acknowledgments

- [Iced](https://iced.rs/)
- [hidapi](https://github.com/libusb/hidapi)
- [Best-README-Template](https://github.com/othneildrew/Best-README-Template)
- [devicePEQ](https://github.com/jeromeof/devicePEQ) for reverse-engineered DAC protocols

[contributors-shield]: https://img.shields.io/badge/contributors-2-blue?style=flat&logo=github
[contributors-url]: https://github.com/bukutsu/frost-tune/graphs/contributors
[forks-shield]: https://img.shields.io/badge/forks-0-blue?style=flat&logo=github
[forks-url]: https://github.com/bukutsu/frost-tune/network/members
[stars-shield]: https://img.shields.io/badge/stars-2-brightgreen?style=flat&logo=github
[stars-url]: https://github.com/bukutsu/frost-tune/stargazers
[issues-shield]: https://img.shields.io/badge/issues-1%20open-important?style=flat&logo=github
[issues-url]: https://github.com/bukutsu/frost-tune/issues
[license-shield]: https://img.shields.io/badge/license-MIT-brightgreen?style=flat&logo=github
[license-url]: https://github.com/bukutsu/frost-tune/blob/main/LICENSE
[product-screenshot]: assets/screenshot.png
