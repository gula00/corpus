// CS110L Lecture 5: Error Handling

fn feeling_lucky(input: u32) -> Option<String> {
    if input > 10 {
        Some(String::from("I'm feeling lucky!"))
    } else {
        None
    }
}

// ? 也可以用于 Option：None 时提前返回 None
fn feeling_extra_lucky(input: u32) -> Option<String> {
    let msg = feeling_lucky(input)?; // None 则直接返回 None
    Some(msg + " Are you?")
}

fn main() {
    // === match 处理 ===
    println!("=== match ===");
    match feeling_lucky(20) {
        Some(msg) => println!("有值: {}", msg),
        None => println!("没值"),
    }
    match feeling_lucky(5) {
        Some(msg) => println!("有值: {}", msg),
        None => println!("没值"),
    }

    // === is_some / is_none 判断 ===
    println!("\n=== is_some / is_none ===");
    let result = feeling_lucky(20);
    println!("is_some: {}", result.is_some());
    let result = feeling_lucky(5);
    println!("is_none: {}", result.is_none());

    // === unwrap / expect ===
    println!("\n=== unwrap ===");
    let msg = feeling_lucky(20).unwrap();
    println!("unwrap: {}", msg);
    // feeling_lucky(5).unwrap(); // 这里会 panic!

    // === unwrap_or 提供默认值 ===
    println!("\n=== unwrap_or ===");
    let msg = feeling_lucky(5).unwrap_or(String::from("Not lucky :("));
    println!("unwrap_or: {}", msg);

    // === ? 操作符用于 Option ===
    println!("\n=== ? with Option ===");
    match feeling_extra_lucky(20) {
        Some(msg) => println!("extra lucky: {}", msg),
        None => println!("not lucky at all"),
    }
    match feeling_extra_lucky(5) {
        Some(msg) => println!("extra lucky: {}", msg),
        None => println!("not lucky at all"),
    }
}
