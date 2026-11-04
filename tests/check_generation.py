"""Exercise the real cargo-generate interface without building or flashing."""

from pathlib import Path
import json
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
IDENTITY = {
    "chip": "nrf52840",
    "mode": "peripheral",
    "advertised_name": "Test Peripheral",
    "gatt_name": "Test GATT",
    "device_address": "FF:E4:05:1A:8F:FF",
}


def main():
    with tempfile.TemporaryDirectory(prefix="nrf-generation-") as temporary:
        destination = Path(temporary)
        count = 0

        def generate(values, error=None):
            nonlocal count
            count += 1
            name = f"check-{count}"
            command = [
                "cargo", "generate", "--path", str(ROOT), "--name", name,
                "--destination", str(destination), "--silent", "--vcs", "none",
            ]
            if any(isinstance(value, str) and "\n" in value for value in values.values()):
                values_file = destination / f"{name}.toml"
                values_file.write_text("[values]\n" + "\n".join(
                    f"{key} = {json.dumps(value, ensure_ascii=False)}"
                    for key, value in values.items()
                ))
                command.extend(["--values-file", str(values_file)])
            else:
                for key, value in values.items():
                    if isinstance(value, bool):
                        value = str(value).lower()
                    command.extend(["--define", f"{key}={value}"])
            result = subprocess.run(command, capture_output=True, text=True)
            output = result.stdout + result.stderr
            if error is not None:
                assert result.returncode != 0, output
                assert error.lower() in output.lower(), output
                return None
            assert result.returncode == 0, output
            project = destination / name
            assert not (project / ".template").exists()
            assert not (project / "cargo-generate.toml").exists()
            assert not (project / "tests").exists()
            assert not (project / "docs").exists()
            assert f'name = "{name}"' in (project / "Cargo.toml").read_text()
            assert f'name = "{name}"' in (project / "Cargo.lock").read_text()
            for path in ["Cargo.toml", "src/main.rs", ".cargo/config.toml"]:
                text = (project / path).read_text()
                assert "{{" not in text and "{%" not in text, path
            return project

        scanner = generate({"chip": "nrf52840", "mode": "scanner"})
        assert (scanner / "src/ble_scanner.rs").read_bytes() == (ROOT / "src/ble_scanner.rs").read_bytes()
        assert not (scanner / "src/ble_peripheral.rs").exists()
        assert not (scanner / "src/csprng.rs").exists()
        assert "STORAGE" not in (scanner / "memory.x").read_text()

        peripheral = generate(IDENTITY)
        assert not (peripheral / "src/ble_scanner.rs").exists()
        assert (peripheral / "src/csprng.rs").exists()
        assert "LENGTH = 1016K" in (peripheral / "memory.x").read_text()
        source = (peripheral / "src/ble_peripheral.rs").read_text()
        assert "[0xff, 0x8f, 0x1a, 0x5, 0xe4, 0xff]" in source

        for project in [scanner, peripheral]:
            assert "thumbv7em-none-eabi" in (project / ".cargo/config.toml").read_text()
            assert '"nrf52840"' in (project / "Cargo.toml").read_text()
            assert 'cortex-m = "0.7.6"\n' in (project / "Cargo.toml").read_text()
            assert not (project / "src/xiao.rs").exists()

        nrf54 = {"chip": "nrf54l15", "xiao_rf_switch": False}
        for xiao in [False, True]:
            values = nrf54 | {"xiao_rf_switch": xiao}
            scanner = generate(values | {"mode": "scanner"})
            peripheral = generate(IDENTITY | values)
            assert (scanner / "src/ble_scanner.rs").read_bytes() == (ROOT / "src/ble_scanner.rs").read_bytes()
            assert not (scanner / "src/ble_peripheral.rs").exists()
            assert not (peripheral / "src/ble_scanner.rs").exists()
            assert (peripheral / "src/csprng.rs").exists()
            assert "LENGTH = 1524K" in (scanner / "memory.x").read_text()
            assert "STORAGE" not in (scanner / "memory.x").read_text()
            memory = (peripheral / "memory.x").read_text()
            assert "LENGTH = 1516K" in memory and "ORIGIN = 0x0017B000, LENGTH = 8K" in memory
            for project in [scanner, peripheral]:
                config = (project / ".cargo/config.toml").read_text()
                assert "thumbv8m.main-none-eabihf" in config and "--chip nRF54L15" in config
                cargo = (project / "Cargo.toml").read_text()
                assert '"nrf54l15-app-s"' in cargo and 'cortex-m = "0.7.6"\n' in cargo
                assert ('"unstable-pac"' in cargo) == xiao
                assert (project / "src/xiao.rs").exists() == xiao
                assert ("mod xiao;" in (project / "src/main.rs").read_text()) == xiao

        generate({"mode": "scanner"}, "chip")
        generate({"chip": "invalid", "mode": "scanner"}, "invalid")
        generate({"chip": "nrf54l15", "mode": "scanner"}, "xiao_rf_switch")
        generate({"chip": "nrf52840"}, "mode")
        generate({"chip": "nrf52840", "mode": "invalid"}, "invalid")
        for key in ["advertised_name", "gatt_name", "device_address"]:
            generate({k: v for k, v in IDENTITY.items() if k != key}, key)
        for key in ["advertised_name", "gatt_name"]:
            for value in ["", "x" * 23, "é" * 12]:
                generate(IDENTITY | {key: value}, key)
            for value in ["x" * 22, "é" * 11, 'A"\\\n{{x}}🦀']:
                project = generate(IDENTITY | {key: value})
                text = (project / "src/ble_peripheral.rs").read_text()
                escaped = "".join(
                    "\\u{" + format(ord(ch), "x") + "}"
                    if ord(ch) < 32 or ord(ch) in [127, 34, 92, 123, 125] else ch
                    for ch in value
                )
                assert f'"{escaped}"' in text
                decoded = re.sub(r"\\u\{([0-9a-f]+)\}", lambda m: chr(int(m[1], 16)), escaped)
                assert decoded == value
        for value in ["FF:E4:05:1A:8F", "FF:E4:05:1A:8F:GG", "F:E4:05:1A:8F:FF", "BF:E4:05:1A:8F:FF", "C0:00:00:00:00:00", "FF:FF:FF:FF:FF:FF"]:
            generate(IDENTITY | {"device_address": value}, "address")
        generate(IDENTITY | {"device_address": "c0:00:00:00:00:01"})
        generate(IDENTITY | {"device_address": "ff:ff:ff:ff:ff:fe"})
        print(f"Passed {count} generation scenarios")


if __name__ == "__main__":
    main()
