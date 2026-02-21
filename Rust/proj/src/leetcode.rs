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
