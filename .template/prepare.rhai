let mode = variable::get("mode");
if mode != "scanner" && mode != "peripheral" {
    throw "mode must be scanner or peripheral";
}

if mode == "peripheral" {
    for key in ["advertised_name", "gatt_name"] {
        let value = variable::get(key);
        let bytes = value.to_blob();
        if bytes.len() == 0 || bytes.len() > 22 {
            throw key + " must contain 1–22 UTF-8 bytes";
        }
        // Escape special characters while keeping ordinary names readable.
        let escaped = "";
        for ch in value.chars() {
            let code = ch.to_int();
            if code < 32 || code == 127 || code == 34 || code == 92 || code == 123 || code == 125 {
                escaped += "\\u{" + code.to_hex() + "}";
            } else {
                escaped += ch;
            }
        }
        variable::set(key + "_rust", escaped);
    }

    let address = variable::get("device_address");
    let parts = address.split(":");
    if parts.len() != 6 {
        throw "device_address must use AA:BB:CC:DD:EE:FF notation";
    }
    let bytes = [];
    for part in parts {
        if part.len() != 2 {
            throw "device_address must contain six two-digit hexadecimal octets";
        }
        let byte = 0;
        for ch in part.to_lower().chars() {
            let digit = "0123456789abcdef".index_of(ch);
            if digit < 0 { throw "device_address contains a non-hexadecimal digit"; }
            byte = byte * 16 + digit;
        }
        bytes.push(byte);
    }
    if bytes[0] < 192 {
        throw "static-random addresses must have their two highest bits set (first octet C0–FF)";
    }
    let all_zero = (bytes[0] & 63) == 0;
    let all_one = (bytes[0] & 63) == 63;
    for i in 1..6 {
        all_zero = all_zero && bytes[i] == 0;
        all_one = all_one && bytes[i] == 255;
    }
    if all_zero || all_one {
        throw "the random portion of device_address cannot be all zeros or all ones";
    }
    bytes.reverse();
    let encoded = "";
    for byte in bytes {
        if encoded != "" { encoded += ", "; }
        encoded += "0x" + byte.to_hex();
    }
    variable::set("device_address_bytes", encoded);

    for path in ["Cargo.toml", "Cargo.lock", "memory.x", "src/main.rs"] {
        file::delete(path);
        file::rename(".template/peripheral/" + path, path);
    }
    for name in ["ble_peripheral.rs", "csprng.rs"] {
        file::rename(".template/peripheral/src/" + name, "src/" + name);
    }
    file::delete("src/ble_scanner.rs");
}
file::delete(".template");
