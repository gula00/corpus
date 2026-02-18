// CS110L Lecture 4: Ownership Practice

// 1. 所有权转移：String 传参后原变量失效
fn om_nom_nom(param: String) {
    println!("{}", param);
}

// 2. Copy trait：基本类型传参是复制
fn om_nom_nom_u32(param: u32) {
    println!("{}", param);
}

fn main() {
    // === 不可变性 ===
    let mut s = String::from("hello");
    s.push_str(" world");
    println!("1. mutability: {}", s);

    // === 所有权转移 ===
    let s2 = String::from("hello");
    om_nom_nom(s2);
    // om_nom_nom(s2); // ERROR: s2 already moved

    // === Copy trait ===
    let x = 1;
    om_nom_nom_u32(x);
    om_nom_nom_u32(x); // OK: u32 is Copy

    // === 借用规则 ===
    let mut s3 = String::from("hello");
    let r = &mut s3;
    println!("3. mutable ref: {}", r); // r 的最后使用
    println!("3. original: {}", s3);   // r 已释放，s3 可用

    // === 方法1: &mut 遍历，修改已有元素 ===
    let mut v = vec![1, 2, 3];
    for i in &mut v {
        *i *= 2; // 解引用后修改元素的值
    }
    println!("5. after *2: {:?}", v); // [2, 4, 6]

    // === 方法2: 用下标遍历，可以增删 ===
    let mut v2 = vec![1, 2, 3];
    let mut idx = 0;
    while idx < v2.len() {
        if v2[idx] == 2 {
            v2.remove(idx); // 删除元素，不推进 idx
        } else {
            idx += 1;
        }
    }
    println!("6. after remove 2: {:?}", v2); // [1, 3]

    // === 方法3: retain — 过滤式删除 ===
    let mut v3 = vec![1, 2, 3, 4, 5];
    v3.retain(|&x| x % 2 != 0); // 只保留奇数
    println!("7. retain odd: {:?}", v3); // [1, 3, 5]

    // === 方法4: iter + collect 生成新集合 ===
    let v4 = vec![1, 2, 3];
    let v5: Vec<i32> = v4.iter().map(|x| x * 10).collect();
    println!("8. map *10: {:?}", v5); // [10, 20, 30]

    // === 方法5: 先收集要改的信息，遍历后再改 ===
    let mut v6 = vec![1, 2, 3];
    let to_add: Vec<i32> = v6.iter().map(|x| x + 10).collect();
    for val in to_add {
        v6.push(val);
    }
    println!("9. deferred push: {:?}", v6); // [1, 2, 3, 11, 12, 13]
}
