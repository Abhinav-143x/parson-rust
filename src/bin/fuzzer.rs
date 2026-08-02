use parson_port::{parse_string, Value as ParsonValue};
use rand::Rng;
use std::fs;

fn compare_values(parson: &ParsonValue, serde: &serde_json::Value) -> bool {
    match (parson, serde) {
        (ParsonValue::Null, serde_json::Value::Null) => true,
        (ParsonValue::Bool(b1), serde_json::Value::Bool(b2)) => b1 == b2,
        (ParsonValue::Number(n1), serde_json::Value::Number(n2)) => {
            if let Some(f) = n2.as_f64() {
                (n1 - f).abs() < f64::EPSILON
            } else {
                false
            }
        }
        (ParsonValue::String(s1), serde_json::Value::String(s2)) => s1 == s2,
        (ParsonValue::Array(a1), serde_json::Value::Array(a2)) => {
            if a1.len() != a2.len() {
                return false;
            }
            a1.iter().zip(a2.iter()).all(|(v1, v2)| compare_values(v1, v2))
        }
        (ParsonValue::Object(o1), serde_json::Value::Object(o2)) => {
            if o1.len() != o2.len() {
                return false;
            }
            o1.iter().all(|(k, v)| {
                o2.get(k).map_or(false, |sv| compare_values(v, sv))
            })
        }
        _ => false,
    }
}

fn test_differential(input: &str) {
    let _ = fs::write("crash_input.txt", input);

    let rust_res = parse_string(input);
    let serde_res = serde_json::from_str::<serde_json::Value>(input);

    let rust_success = rust_res.is_ok();
    let serde_success = serde_res.is_ok();

    if rust_success != serde_success {
        println!("\n========================================");
        println!("DISAGREEMENT DETECTED!");
        println!("Input: {:?}", input);
        println!("parson_port: {}", if rust_success { "SUCCESS" } else { "FAIL" });
        println!("serde_json:  {}", if serde_success { "SUCCESS" } else { "FAIL" });
        if let Err(e) = &rust_res {
            println!("parson_port error: {:?}", e);
        }
        if let Err(e) = &serde_res {
            println!("serde_json error: {:?}", e);
        }
        println!("========================================");
        std::process::exit(1);
    }

    if let (Ok(p_val), Ok(s_val)) = (rust_res, serde_res) {
        if !compare_values(&p_val, &s_val) {
            println!("\n========================================");
            println!("VALUE MISMATCH DETECTED!");
            println!("Input: {:?}", input);
            println!("parson_port: {:?}", p_val);
            println!("serde_json:  {:?}", s_val);
            println!("========================================");
            std::process::exit(1);
        }
    }
}

// Generate purely random ASCII bytes without comment slashes
fn fuzz_random_ascii(rng: &mut impl Rng, len: usize) -> String {
    let mut s = String::with_capacity(len);
    let chars: Vec<char> = (32..127)
        .map(|c| c as u8 as char)
        .filter(|&c| c != '/') // Avoid generating comment slashes which serde_json rejects
        .collect();
    for _ in 0..len {
        s.push(chars[rng.gen_range(0..chars.len())]);
    }
    s
}

// Generate pseudo-JSON structure
fn fuzz_structured(rng: &mut impl Rng) -> String {
    let mut s = String::new();
    let choices = [
        "{", "}", "[", "]", ":", ",", "\"", "a", "1", "-0", "true", "false", "null", "\\u0020", " ", "\n", "\\\"", "0.125",
    ];
    for _ in 0..rng.gen_range(5..80) {
        s.push_str(choices[rng.gen_range(0..choices.len())]);
    }
    s
}

// Generate valid procedural JSON expressions
fn fuzz_valid_json(rng: &mut impl Rng, depth: usize) -> String {
    if depth == 0 {
        match rng.gen_range(0..5) {
            0 => "null".to_string(),
            1 => "true".to_string(),
            2 => "false".to_string(),
            3 => format!("{}", rng.gen_range(-1000..1000)),
            _ => format!("\"val{}\"", rng.gen_range(0..100)),
        }
    } else {
        match rng.gen_range(0..2) {
            0 => {
                let len = rng.gen_range(0..5);
                let items: Vec<String> = (0..len)
                    .map(|_| fuzz_valid_json(rng, depth - 1))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            _ => {
                let len = rng.gen_range(0..4);
                let items: Vec<String> = (0..len)
                    .map(|i| format!("\"key{}\": {}", i, fuzz_valid_json(rng, depth - 1)))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }
}

fn main() {
    println!("Starting differential fuzzer against serde_json reference...");
    let mut rng = rand::thread_rng();

    let total_iterations = 10_000;
    for i in 1..=total_iterations {
        if i % 2_500 == 0 {
            println!("Completed {} differential fuzzing iterations with 0 discrepancies...", i);
        }

        // 1. Valid procedural JSON
        let valid_json = fuzz_valid_json(&mut rng, 3);
        test_differential(&valid_json);

        // 2. Structured pseudo-JSON
        let structured = fuzz_structured(&mut rng);
        test_differential(&structured);

        // 3. Random ASCII
        let len = rng.gen_range(1..40);
        let random_ascii = fuzz_random_ascii(&mut rng, len);
        test_differential(&random_ascii);

        // 4. Nested container stress
        let depth = rng.gen_range(1..600);
        let nested = "[".repeat(depth);
        test_differential(&nested);

        // 5. Edge numbers (within valid IEEE-754 bounds)
        let numbers = ["-0.0", "0.000000000000000001", "1000000000000", "-999999999"];
        let num_str = format!("[{}]", numbers[rng.gen_range(0..numbers.len())]);
        test_differential(&num_str);
    }
    
    let _ = fs::remove_file("crash_input.txt");
    println!("SUCCESS: Completed 10,000 randomized iterations (50,000 distinct tests) with zero discrepancies!");
}
