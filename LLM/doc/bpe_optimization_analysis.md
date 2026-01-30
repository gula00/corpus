# BPE Training 优化分析文档
???
But at large scale, it doesn't really matter. The major bottleneck is the pre-tokenization step, which can be optimized by parallelizing the tokenization process.

本文档详细分析 `bpe_train.py` 的性能瓶颈及优化方案。

---

## 目录

1. [原始实现分析](#1-原始实现分析)
2. [性能瓶颈识别](#2-性能瓶颈识别)
3. [优化方案详解](#3-优化方案详解)
4. [数据结构对比](#4-数据结构对比)
5. [复杂度分析](#5-复杂度分析)
6. [代码对比](#6-代码对比)

---

## 1. 原始实现分析

### 1.1 核心数据结构

```python
freqs: dict[tuple[int, ...], int]      # 词 -> 频率
pair_freqs: dict[tuple[int, int], int] # 字节对 -> 频率
```

### 1.2 算法流程

```
1. 预分词 (pre_tokenize)
   └── 并行处理文件，统计词频

2. 构建字节对频率表 (get_pair_freqs)
   └── 遍历所有词，统计相邻字节对频率

3. 迭代合并 (merge loop)
   ├── 查找最高频字节对: max(pair_freqs, key=...)  ← 瓶颈1
   ├── 执行合并: merge(freqs, pair_freqs, a, b)   ← 瓶颈2
   └── 重复直到达到目标词表大小
```

---

## 2. 性能瓶颈识别

### 瓶颈 1: 线性扫描查找最大值

**原始代码** (train_bpe 第174行):
```python
best = max(pair_freqs, key=lambda p: (pair_freqs[p], id_to_bytes[p[0]], id_to_bytes[p[1]]))
```

**问题分析**:
- 每次迭代都要遍历整个 `pair_freqs` 字典
- 设 P = 字节对数量，V = 目标合并次数
- 总复杂度: **O(V × P)**

**实际影响**:
- P 通常在 10^5 ~ 10^6 量级
- V 通常在 10^4 ~ 10^5 量级 (vocab_size - 256)
- 总操作数: 10^9 ~ 10^11

### 瓶颈 2: 遍历所有词进行合并

**原始代码** (merge 函数):
```python
for word, freq in freqs.items():                    # 遍历所有词
    if (a, b) not in zip(word, word[1:]):           # 检查是否包含目标对
        ...
```

**问题分析**:
- 每次合并都要遍历所有 N 个词
- 即使大部分词不包含目标字节对，也要检查
- 设 L = 平均词长度
- 单次合并复杂度: **O(N × L)**
- 总复杂度: **O(V × N × L)**

**实际影响**:
- N (唯一词数) 通常在 10^5 ~ 10^7 量级
- L (平均词长) 通常在 5 ~ 20
- 大量无效遍历

---

## 3. 优化方案详解

### 3.1 优化 1: 最大堆 (Max Heap)

**思路**: 用堆维护字节对频率，O(1) 获取最大值

**实现要点**:

```python
# Python heapq 是最小堆，用负数模拟最大堆
# 堆元素: (-freq, ReversedBytes_a, ReversedBytes_b, pair)

self.heap: list[tuple[int, _ReversedBytes, _ReversedBytes, tuple[int, int]]] = []

def _heap_push(self, pair: tuple[int, int], freq: int):
    a, b = pair
    heapq.heappush(
        self.heap,
        (-freq, _ReversedBytes(self.id_to_bytes[a]), _ReversedBytes(self.id_to_bytes[b]), pair)
    )
```

**关键细节 - 字典序反转**:

原始实现使用 `max()` 在频率相同时选择**字典序最大**的 bytes：
```python
best = max(pair_freqs, key=lambda p: (pair_freqs[p], id_to_bytes[p[0]], id_to_bytes[p[1]]))
```

但 Python `heapq` 是最小堆，会选择**字典序最小**的。需要用包装类反转比较：

```python
class _ReversedBytes:
    """Wrapper for bytes to reverse comparison order."""
    __slots__ = ('b',)

    def __init__(self, b: bytes):
        self.b = b

    def __lt__(self, other):
        return self.b > other.b  # 反转: 大的排前面

    def __eq__(self, other):
        return self.b == other.b
```

这样堆在频率相同时会优先弹出字典序较大的 bytes，与原始 `max()` 行为一致。

**懒删除机制**:

频率会动态变化，不能直接删除堆中元素。采用懒删除：
- 不从堆中物理删除过期条目
- 取堆顶时检查是否有效
- 无效则弹出并继续

```python
def get_best_pair(self) -> tuple[int, int] | None:
    while self.heap:
        neg_freq, _, _, pair = self.heap[0]
        actual_freq = self.pair_freqs.get(pair, 0)

        # 检查是否过期 (频率变化或已删除)
        if actual_freq <= 0 or actual_freq != -neg_freq:
            heapq.heappop(self.heap)  # 懒删除
            continue

        heapq.heappop(self.heap)
        return pair
    return None
```

### 3.2 优化 2: 倒排索引 (Inverted Index)

**思路**: 维护 `pair -> words` 映射，只处理相关词

**数据结构**:

```python
# 倒排索引: 字节对 -> 包含该字节对的词集合
self.pair_to_words: dict[tuple[int, int], set[tuple[int, ...]]] = defaultdict(set)
```

**构建索引**:

```python
def _build_index(self):
    for word, freq in self.freqs.items():
        for i in range(len(word) - 1):
            pair = (word[i], word[i + 1])
            self.pair_freqs[pair] += freq
            self.pair_to_words[pair].add(word)  # 建立倒排索引
```

**使用索引进行合并**:

```python
def merge(self, a: int, b: int, new_id: int):
    target_pair = (a, b)

    # 关键优化: 只获取包含目标对的词
    words_to_process = self.pair_to_words.pop(target_pair, set()).copy()

    for word in words_to_process:  # 只遍历相关词，而非全部词
        # ... 执行合并 ...
```

### 3.3 索引维护

合并后需要更新索引:

```python
# 处理每个受影响的词
for word in words_to_process:
    freq = self.freqs.pop(word)

    # 1. 从倒排索引移除旧词的所有对
    for i in range(len(word) - 1):
        pair = (word[i], word[i + 1])
        self.pair_to_words[pair].discard(word)
        self.pair_freqs[pair] -= freq

    # 2. 构建新词 (执行合并)
    new_word = build_merged_word(word, a, b, new_id)

    # 3. 添加新词的所有对到倒排索引
    for i in range(len(new_word) - 1):
        pair = (new_word[i], new_word[i + 1])
        self.pair_to_words[pair].add(new_word)
        self.pair_freqs[pair] += freq

    # 4. 更新堆
    for affected_pair in affected_pairs:
        if self.pair_freqs[affected_pair] > 0:
            self._heap_push(affected_pair, self.pair_freqs[affected_pair])
```

---

## 4. 数据结构对比

### 4.1 原始实现

```
┌─────────────────────────────────────────────────────┐
│                    数据结构                          │
├─────────────────────────────────────────────────────┤
│  freqs: dict[word -> freq]                          │
│    ┌──────────┬───────┐                             │
│    │   词     │ 频率   │                             │
│    ├──────────┼───────┤                             │
│    │ (1,2,3)  │  100  │                             │
│    │ (1,2,4)  │   50  │                             │
│    │ (5,6,7)  │   30  │                             │
│    └──────────┴───────┘                             │
│                                                     │
│  pair_freqs: dict[pair -> freq]                     │
│    ┌─────────┬───────┐                              │
│    │  字节对  │ 频率   │                              │
│    ├─────────┼───────┤                              │
│    │  (1,2)  │  150  │  ← 需要遍历找最大值           │
│    │  (2,3)  │  100  │                              │
│    │  (2,4)  │   50  │                              │
│    └─────────┴───────┘                              │
└─────────────────────────────────────────────────────┘

查找 (1,2) 时:
  → 遍历 pair_freqs 找最大 O(P)
  → 遍历 freqs 所有词检查是否包含 (1,2) O(N)
```

### 4.2 优化后实现

```
┌─────────────────────────────────────────────────────┐
│                    数据结构                          │
├─────────────────────────────────────────────────────┤
│  freqs: dict[word -> freq]     (同原始)             │
│                                                     │
│  pair_freqs: dict[pair -> freq] (同原始)            │
│                                                     │
│  ★ pair_to_words: dict[pair -> set[word]]  倒排索引 │
│    ┌─────────┬────────────────────┐                 │
│    │  字节对  │     词集合          │                 │
│    ├─────────┼────────────────────┤                 │
│    │  (1,2)  │ {(1,2,3), (1,2,4)} │  ← 直接定位     │
│    │  (2,3)  │ {(1,2,3)}          │                 │
│    │  (2,4)  │ {(1,2,4)}          │                 │
│    └─────────┴────────────────────┘                 │
│                                                     │
│  ★ heap: list[(-freq, bytes, bytes, pair)]  最大堆  │
│                  (-150, b'1', b'2', (1,2))          │
│                 /                    \              │
│    (-100, b'2', b'3', (2,3))    (-50, b'2', b'4',..)│
│                                                     │
└─────────────────────────────────────────────────────┘

查找 (1,2) 时:
  → 堆顶直接获取 O(1)
  → 倒排索引直接获取相关词 O(K), K << N
```

---

## 5. 复杂度分析

### 5.1 时间复杂度

| 操作 | 原始实现 | 优化实现 |
|------|---------|---------|
| 查找最大频率对 | O(P) | O(1) 摊销* |
| 单次合并 | O(N × L) | O(K × L) |
| 总复杂度 | O(V × (P + N × L)) | O(V × log P + Σ K × L) |

其中:
- P = 字节对数量
- N = 唯一词数量
- L = 平均词长度
- V = 合并次数 (vocab_size - 256)
- K = 包含特定对的词数量 (K << N)

*懒删除导致额外的 log P 操作，但摊销后仍为 O(1)

### 5.2 空间复杂度

| 数据结构 | 原始实现 | 优化实现 |
|---------|---------|---------|
| freqs | O(N × L) | O(N × L) |
| pair_freqs | O(P) | O(P) |
| pair_to_words | - | O(P + N × L) |
| heap | - | O(P × log factor) |
| **总计** | O(N × L + P) | O(N × L + P) |

优化实现的空间复杂度增加约 2x，但换取了显著的时间提升。

### 5.3 实际性能预期

假设:
- N = 1,000,000 (唯一词)
- P = 500,000 (字节对)
- V = 50,000 (合并次数)
- K = 1,000 (平均每个对涉及的词数)

| 指标 | 原始实现 | 优化实现 | 加速比 |
|-----|---------|---------|-------|
| 查找操作 | 50,000 × 500,000 = 2.5×10^10 | 50,000 × 1 = 5×10^4 | ~500,000x |
| 合并操作 | 50,000 × 1,000,000 = 5×10^10 | 50,000 × 1,000 = 5×10^7 | ~1,000x |

---

## 6. 代码对比

### 6.1 查找最大频率对

**原始**:
```python
# O(P) - 每次都遍历整个字典
best = max(pair_freqs, key=lambda p: (pair_freqs[p], id_to_bytes[p[0]], id_to_bytes[p[1]]))
```

**优化**:
```python
# O(1) 摊销 - 堆顶直接获取
def get_best_pair(self) -> tuple[int, int] | None:
    while self.heap:
        neg_freq, _, _, pair = self.heap[0]
        actual_freq = self.pair_freqs.get(pair, 0)
        if actual_freq <= 0 or actual_freq != -neg_freq:
            heapq.heappop(self.heap)
            continue
        heapq.heappop(self.heap)
        return pair
    return None
```

### 6.2 合并操作

**原始**:
```python
# O(N × L) - 遍历所有词
def merge(freqs, pair_freqs, a, b, new_id):
    new_freqs = {}
    for word, freq in freqs.items():  # 遍历全部词
        if (a, b) not in zip(word, word[1:]):
            new_freqs[word] = freq
            continue
        # ... 执行合并 ...
    return new_freqs
```

**优化**:
```python
# O(K × L) - 只处理相关词
def merge(self, a: int, b: int, new_id: int):
    target_pair = (a, b)
    # 通过倒排索引直接获取相关词
    words_to_process = self.pair_to_words.pop(target_pair, set()).copy()

    for word in words_to_process:  # 只遍历包含目标对的词
        # ... 执行合并 ...
```

### 6.3 完整优化类结构

```python
class _ReversedBytes:
    """反转 bytes 比较顺序，保证与原始 max() 行为一致"""
    __slots__ = ('b',)

    def __init__(self, b: bytes):
        self.b = b

    def __lt__(self, other):
        return self.b > other.b  # 反转比较


class BPETrainer:
    def __init__(self, freqs, id_to_bytes):
        self.freqs = dict(freqs)
        self.id_to_bytes = id_to_bytes
        self.pair_freqs = defaultdict(int)
        self.pair_to_words = defaultdict(set)  # 倒排索引
        self.heap = []                          # 最大堆
        self._build_index()

    def _build_index(self):
        """构建倒排索引和堆"""
        for word, freq in self.freqs.items():
            for i in range(len(word) - 1):
                pair = (word[i], word[i + 1])
                self.pair_freqs[pair] += freq
                self.pair_to_words[pair].add(word)
        for pair, freq in self.pair_freqs.items():
            if freq > 0:
                self._heap_push(pair, freq)

    def _heap_push(self, pair, freq):
        """推入堆，使用 _ReversedBytes 保证排序正确"""
        a, b = pair
        heapq.heappush(self.heap, (
            -freq,
            _ReversedBytes(self.id_to_bytes[a]),
            _ReversedBytes(self.id_to_bytes[b]),
            pair
        ))

    def get_best_pair(self):
        """O(1) 摊销获取最大频率对，懒删除过期条目"""
        while self.heap:
            neg_freq, _, _, pair = self.heap[0]
            actual_freq = self.pair_freqs.get(pair, 0)
            if actual_freq <= 0 or actual_freq != -neg_freq:
                heapq.heappop(self.heap)  # 过期，删除
                continue
            heapq.heappop(self.heap)
            return pair
        return None

    def merge(self, a, b, new_id):
        """O(K × L) 合并，K = 受影响词数"""
        words_to_process = self.pair_to_words.pop((a, b), set()).copy()
        # ... 只处理 words_to_process 中的词 ...
```

---

## 总结

| 优化技术 | 解决的问题 | 效果 |
|---------|-----------|------|
| **最大堆** | 避免线性扫描找最大值 | O(P) → O(1) |
| **倒排索引** | 避免遍历无关词 | O(N) → O(K) |
| **懒删除** | 高效处理频率变化 | 避免堆重建 |

总体加速比预期: **100x ~ 1000x** (取决于数据规模)
