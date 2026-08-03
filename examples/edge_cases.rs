use parson_port::{parse_string, serialize_to_string};

fn main() {
    println!("=== NUMBER PARSING EDGE CASES ===");
    let parse_cases = vec![
        "0e1", "-0e1", "0E5", "0.0e1", "-0.0e1", 
        "00", "01", "-01", "-00",
        "0", "-0", "0.0", "-0.0",
        "1.", "1.e5",    // trailing dot
        ".5",            // leading dot
    ];
    for c in &parse_cases {
        println!("{:10} => {:?}", c, parse_string(c));
    }

    println!("\n=== SERIALIZATION ROUND-TRIP ===");
    let ser_cases = vec![
        0.0_f64, -0.0, 1.0, -1.0, 42.5, 0.1, 
        1e20, 1e-5, 1e17, 1e-4, 123456789.0,
        0.001, 1.0/3.0, 1e100, 1e-100,
        f64::MAX, f64::MIN_POSITIVE,
    ];
    for n in &ser_cases {
        let val = parson_port::Value::Number(*n);
        let s = serialize_to_string(&val);
        println!("{:25e} => {}", n, s);
    }

    println!("\n=== UTF-8 BOM TEST ===");
    let bom = "\u{FEFF}{\"a\":1}";
    println!("BOM + json => {:?}", parse_string(bom));
    
    println!("\n=== CONTROL CHAR BOUNDARY ===");
    // C checks: (unsigned char)*input_ptr < 0x20
    // Our Rust checks: c < ' ' (i.e., c < 0x20)
    // 0x1F should be rejected, 0x20 (space) should be ok
    let s_with_0x1f = format!("[\"\\x{:02x}\"]", 0x1Fu8);  // can't embed directly
    println!("String with 0x1F char embedded: {:?}", parse_string(&format!("[\"\x1f\"]")));
    println!("String with space (0x20): {:?}", parse_string("[\" \"]"));
    
    println!("\n=== LONE LOW SURROGATE ===");
    println!("Lone low surrogate \\uDC00: {:?}", parse_string("[\"\\uDC00\"]"));
    println!("Lone high surrogate \\uD800: {:?}", parse_string("[\"\\uD800\"]"));
}
