/// 88. Merge Sorted Array
/// 从后往前双指针归并，原地合并两个有序数组
pub fn merge(nums1: &mut Vec<i32>, m: i32, nums2: &mut Vec<i32>, n: i32) {
    let (mut i, mut j, mut k) = (m as usize, n as usize, (m + n) as usize);
    while j > 0 {
        k -= 1;
        if i > 0 && nums1[i - 1] > nums2[j - 1] {
            nums1[k] = nums1[i - 1];
            i -= 1;
        } else {
            nums1[k] = nums2[j - 1];
            j -= 1;
        }
    }
}

/// 27. Remove Element
/// 快慢双指针，原地移除所有等于 val 的元素，返回剩余长度
pub fn remove_element(nums: &mut Vec<i32>, val: i32) -> i32 {
    let mut slow = 0;
    for fast in 0..nums.len() {
        if nums[fast] != val {
            nums[slow] = nums[fast];
            slow += 1;
        }
    }
    slow as i32
}

/// 26. Remove Duplicates from Sorted Array
/// 快慢双指针，跳过重复元素，返回去重后长度
pub fn remove_duplicates(nums: &mut Vec<i32>) -> i32 {
    if nums.is_empty() {
        return 0;
    }
    let mut slow = 0;
    for fast in 1..nums.len() {
        if nums[fast] != nums[slow] {
            slow += 1;
            nums[slow] = nums[fast];
        }
    }
    (slow + 1) as i32
}

/// 80. Remove Duplicates from Sorted Array II
/// 快慢双指针，每个元素最多保留两次
pub fn remove_duplicates_ii(nums: &mut Vec<i32>) -> i32 {
    if nums.len() <= 2 {
        return nums.len() as i32;
    }
    let mut slow = 2;
    for fast in 2..nums.len() {
        if nums[fast] != nums[slow - 2] {
            nums[slow] = nums[fast];
            slow += 1;
        }
    }
    slow as i32
}

/// 169. Majority Element
/// Boyer-Moore 投票算法，找出出现次数超过 n/2 的元素 (打擂台)
pub fn majority_element(nums: Vec<i32>) -> i32 {
    let mut candidate = 0;
    let mut count = 0;
    for &num in &nums {
        if count == 0 {
            candidate = num;
        }
        count += if num == candidate { 1 } else { -1 };
    }
    candidate
}

/// 189. Rotate Array
/// 三次翻转法，原地右轮转 k 步
pub fn rotate(nums: &mut Vec<i32>, k: i32) {
    let n = nums.len();
    let k = k as usize % n;
    nums.reverse();
    nums[..k].reverse();
    nums[k..].reverse();
}

/// 121. Best Time to Buy and Sell Stock
/// 一次遍历：维护历史最低价，并尝试在当天卖出更新最大利润
pub fn max_profit(prices: Vec<i32>) -> i32 {
    let mut min_price = i32::MAX;
    let mut best = 0;

    for price in prices {
        if price < min_price {
            min_price = price;
        } else {
            best = best.max(price - min_price);
        }
    }

    best
}

/// 122. Best Time to Buy and Sell Stock II
/// 贪心：把所有相邻上升区间的收益累加
pub fn max_profit_ii(prices: Vec<i32>) -> i32 {
    let mut profit = 0;
    for i in 1..prices.len() {
        if prices[i] > prices[i - 1] {
            profit += prices[i] - prices[i - 1];
        }
    }
    profit
}

/// 122. Best Time to Buy and Sell Stock II (DP 状态机)
/// f0: 当前不持股的最大利润；f1: 当前持股的最大利润
pub fn max_profit_ii_dp(prices: Vec<i32>) -> i32 {
    let mut f0 = 0;
    let mut f1 = i32::MIN;

    for p in prices {
        let new_f0 = f0.max(f1 + p);
        f1 = f1.max(f0 - p);
        f0 = new_f0;
    }

    f0
}

/// 55. Jump Game
/// 贪心：维护当前能到达的最远下标
pub fn can_jump(nums: Vec<i32>) -> bool {
    let mut farthest = 0usize;

    for (i, &step) in nums.iter().enumerate() {
        if i > farthest {
            return false;
        }
        farthest = farthest.max(i + step as usize);
        if farthest >= nums.len().saturating_sub(1) {
            return true;
        }
    }

    true
}

/// 45. Jump Game II
/// 贪心分层：在当前步数可达区间内，计算下一层最远可达位置
pub fn jump(nums: Vec<i32>) -> i32 {
    if nums.len() <= 1 {
        return 0;
    }

    let mut steps = 0;
    let mut current_end = 0usize;
    let mut farthest = 0usize;

    for (i, &step) in nums.iter().enumerate().take(nums.len() - 1) {
        farthest = farthest.max(i + step as usize);
        if i == current_end {
            steps += 1;
            current_end = farthest;
        }
    }

    steps
}

/// 274. H-Index
/// 排序后枚举：找满足 citations[i] >= n - i 的最大 h
pub fn h_index(mut citations: Vec<i32>) -> i32 {
    citations.sort_unstable();
    let n = citations.len();

    for (i, &c) in citations.iter().enumerate() {
        let h = (n - i) as i32;
        if c >= h {
            return h;
        }
    }

    0
}

use rand::Rng;
use std::collections::HashMap;

/// 380. Insert Delete GetRandom O(1)
/// 用数组存值，用哈希表记录每个值在数组中的下标
pub struct RandomizedSet {
    nums: Vec<i32>,
    pos: HashMap<i32, usize>,
}

impl RandomizedSet {
    pub fn new() -> Self {
        Self {
            nums: Vec::new(),
            pos: HashMap::new(),
        }
    }

    pub fn insert(&mut self, val: i32) -> bool {
        if self.pos.contains_key(&val) {
            return false;
        }
        let idx = self.nums.len();
        self.nums.push(val);
        self.pos.insert(val, idx);
        true
    }

    pub fn remove(&mut self, val: i32) -> bool {
        let Some(&idx) = self.pos.get(&val) else {
            return false;
        };

        let last_idx = self.nums.len() - 1;
        let last_val = self.nums[last_idx];
        self.nums.swap(idx, last_idx);
        self.nums.pop();
        self.pos.remove(&val);

        if idx != last_idx {
            self.pos.insert(last_val, idx);
        }

        true
    }

    pub fn get_random(&self) -> i32 {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.nums.len());
        self.nums[idx]
    }
}

/// 238. Product of Array Except Self
/// 前后缀积：res[i] 先存左侧乘积，再乘上右侧乘积
pub fn product_except_self(nums: Vec<i32>) -> Vec<i32> {
    let n = nums.len();
    let mut res = vec![1; n];

    let mut prefix = 1;
    for i in 0..n {
        res[i] = prefix;
        prefix *= nums[i];
    }

    let mut suffix = 1;
    for i in (0..n).rev() {
        res[i] *= suffix;
        suffix *= nums[i];
    }

    res
}

/// 134. Gas Station
/// 贪心：
/// 1) 总油量 < 总消耗则无解
/// 2) 扫描时一旦当前油量为负，起点更新到下一站
pub fn can_complete_circuit(gas: Vec<i32>, cost: Vec<i32>) -> i32 {
    let total: i32 = gas.iter().sum::<i32>() - cost.iter().sum::<i32>();
    if total < 0 {
        return -1;
    }

    let mut start = 0usize;
    let mut tank = 0;
    for i in 0..gas.len() {
        tank += gas[i] - cost[i];
        if tank < 0 {
            start = i + 1;
            tank = 0;
        }
    }

    start as i32
}

/// 135. Candy
/// 双向贪心：
/// - 从左到右保证右边评分更高时糖果更多
/// - 从右到左保证左边评分更高时糖果更多
pub fn candy(ratings: Vec<i32>) -> i32 {
    let n = ratings.len();
    let mut candies = vec![1; n];

    for i in 1..n {
        if ratings[i] > ratings[i - 1] {
            candies[i] = candies[i - 1] + 1;
        }
    }

    for i in (0..n.saturating_sub(1)).rev() {
        if ratings[i] > ratings[i + 1] {
            candies[i] = candies[i].max(candies[i + 1] + 1);
        }
    }

    candies.iter().sum()
}

/// 42. Trapping Rain Water
/// 双指针：
/// 哪边的当前高度更低，就由那边的最大高度决定可接水量
pub fn trap(height: Vec<i32>) -> i32 {
    if height.is_empty() {
        return 0;
    }

    let (mut l, mut r) = (0usize, height.len() - 1);
    let (mut left_max, mut right_max) = (0, 0);
    let mut ans = 0;

    while l < r {
        if height[l] < height[r] {
            left_max = left_max.max(height[l]);
            ans += left_max - height[l];
            l += 1;
        } else {
            right_max = right_max.max(height[r]);
            ans += right_max - height[r];
            r -= 1;
        }
    }

    ans
}

/// 42. Trapping Rain Water（前后缀分解）
/// left_max[i] / right_max[i] 分别表示 i 左右两侧（含 i）的最高柱
pub fn trap_prefix_suffix(height: Vec<i32>) -> i32 {
    let n = height.len();
    if n == 0 {
        return 0;
    }

    let mut left_max = vec![0; n];
    let mut right_max = vec![0; n];

    left_max[0] = height[0];
    for i in 1..n {
        left_max[i] = left_max[i - 1].max(height[i]);
    }

    right_max[n - 1] = height[n - 1];
    for i in (0..n - 1).rev() {
        right_max[i] = right_max[i + 1].max(height[i]);
    }

    let mut ans = 0;
    for i in 0..n {
        ans += left_max[i].min(right_max[i]) - height[i];
    }

    ans
}

/// 13. Roman to Integer
/// 从右往左扫描：当前值 < 右侧值则减，否则加
pub fn roman_to_int(s: String) -> i32 {
    fn val(c: u8) -> i32 {
        match c {
            b'I' => 1,
            b'V' => 5,
            b'X' => 10,
            b'L' => 50,
            b'C' => 100,
            b'D' => 500,
            b'M' => 1000,
            _ => 0,
        }
    }

    let bytes = s.as_bytes();
    let mut ans = 0;
    let mut prev = 0;

    for &ch in bytes.iter().rev() {
        let cur = val(ch);
        if cur < prev {
            ans -= cur;
        } else {
            ans += cur;
        }
        prev = cur;
    }

    ans
}

/// 12. Integer to Roman
/// 贪心：从大到小尝试匹配，能减就减并追加对应罗马字符
pub fn int_to_roman(mut num: i32) -> String {
    let vals = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
    let syms = [
        "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
    ];

    let mut ans = String::new();
    for i in 0..vals.len() {
        while num >= vals[i] {
            num -= vals[i];
            ans.push_str(syms[i]);
        }
    }

    ans
}

/// 58. Length of Last Word
/// 从后往前：先跳过末尾空格，再统计最后一个单词长度
pub fn length_of_last_word(s: String) -> i32 {
    let bytes = s.as_bytes();
    let mut i = bytes.len();

    while i > 0 && bytes[i - 1] == b' ' {
        i -= 1;
    }

    let mut len = 0;
    while i > 0 && bytes[i - 1] != b' ' {
        len += 1;
        i -= 1;
    }

    len
}

/// 14. Longest Common Prefix
/// 以前缀字符串为基准，不断缩短直到成为每个字符串的前缀
pub fn longest_common_prefix(strs: Vec<String>) -> String {
    if strs.is_empty() {
        return String::new();
    }

    let mut prefix = strs[0].clone();
    for s in strs.iter().skip(1) {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return prefix;
            }
        }
    }

    prefix
}

/// 151. Reverse Words in a String
/// 按空白切分单词，反转后用单个空格拼接
pub fn reverse_words(s: String) -> String {
    let mut words: Vec<&str> = s.split_whitespace().collect();
    words.reverse();
    words.join(" ")
}

/// 6. Zigzag Conversion
/// 按行模拟：维护当前行和方向（向下/向上）; 或者 iter
pub fn convert(s: String, num_rows: i32) -> String {
    let rows_count = num_rows as usize;
    if rows_count <= 1 || s.len() <= rows_count {
        return s;
    }

    let mut rows = vec![String::new(); rows_count];
    let row_ids = (0..rows_count).chain((1..rows_count - 1).rev()).cycle();

    for (ch, row) in s.chars().zip(row_ids) {
        rows[row].push(ch);
    }
    // row_ids.zip(s.chars()).for_each(|(i, c)| rows[i].push(c));

    rows.concat()
}

/// 28. Find the Index of the First Occurrence in a String
/// 直接用标准库查找子串，找不到返回 -1
pub fn str_str(haystack: String, needle: String) -> i32 {
    haystack.find(&needle).map_or(-1, |idx| idx as i32)
}

/// 28. Find the Index of the First Occurrence in a String (KMP 简要实现)
pub fn str_str_kmp(haystack: String, needle: String) -> i32 {
    if needle.is_empty() {
        return 0;
    }

    let s = haystack.as_bytes();
    let p = needle.as_bytes();

    let mut lps = vec![0usize; p.len()];
    {
        let mut j = 0usize;
        for i in 1..p.len() {
            while j > 0 && p[i] != p[j] {
                j = lps[j - 1];
            }
            if p[i] == p[j] {
                j += 1;
            }
            lps[i] = j;
        }
    }

    let mut j = 0usize;
    for (i, &ch) in s.iter().enumerate() {
        while j > 0 && ch != p[j] {
            j = lps[j - 1];
        }
        if ch == p[j] {
            j += 1;
        }
        if j == p.len() {
            return (i + 1 - p.len()) as i32;
        }
    }

    -1
}
