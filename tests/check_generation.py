"""Exercise the real cargo-generate interface without building or flashing."""

from pathlib import Path
import json
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
IDENTITY = {
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
            if any("\n" in value for value in values.values()):
                values_file = destination / f"{name}.toml"
                values_file.write_text("[values]\n" + "\n".join(
                    f"{key} = {json.dumps(value, ensure_ascii=False)}"
                    for key, value in values.items()
                ))
                command.extend(["--values-file", str(values_file)])
            else:
                for key, value in values.items():
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
            assert f'name = "{name}"' in (project / "Cargo.toml").read_text()
            assert f'name = "{name}"' in (project / "Cargo.lock").read_text()
            assert "{{" not in (project / "Cargo.toml").read_text()
            return project

        scanner = generate({"mode": "scanner"})
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

        generate({}, "mode")
        generate({"mode": "invalid"}, "invalid")
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
