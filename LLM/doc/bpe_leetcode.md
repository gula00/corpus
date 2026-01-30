# BPE Training LeetCode 题目集

本文档将 `bpe_train.py` 中的函数改编为 LeetCode 风格的算法题目。

---

## 1. 文件分块边界查找

**难度**: 中等

### 题目描述

给定一个大文件和分块数量，你需要找到将文件分割成多个块的边界位置。每个边界必须对齐到指定的分隔符（split_token）之后。

具体规则：
1. 首先将文件均匀分成 `num_chunks` 块，计算初始边界位置
2. 对于每个中间边界（不包括文件开头0和文件末尾），从该位置开始向后搜索，找到第一个 `split_token` 出现的位置
3. 将边界调整到该 `split_token` 之后（包含分隔符）
4. 返回所有去重并排序后的边界位置

### 示例

```
输入:
file_content = b"hello\nworld\nfoo\nbar\n"  # 文件大小为20字节
num_chunks = 2
split_token = b"\n"

输出: [0, 12, 20]

解释:
- 初始边界: [0, 10, 20]
- 位置10附近搜索"\n"，在位置11找到，边界调整为12
- 最终边界: [0, 12, 20]
```

### 约束条件

- `1 <= num_chunks <= 文件大小`
- `split_token` 长度 >= 1
- 文件大小 <= 10^9 字节

### 函数签名

```python
def find_chunk_boundaries(file: BinaryIO, num_chunks: int, split_token: bytes) -> list[int]:
    pass
```

---

## 2. 词频统计

**难度**: 简单

### 题目描述

给定一段文本和一个正则表达式模式，对文本进行分词，然后将每个词转换为 UTF-8 字节序列表示的元组，统计每个元组出现的频率。

转换规则：
1. 使用正则表达式 `PAT` 对文本进行分词
2. 对于每个词，将其编码为 UTF-8 字节
3. 每个字节值加上偏移量 `num_special`
4. 将结果存储为元组，统计频率

### 示例

```
输入:
text = "ab ab cd"
num_special = 0
PAT 匹配结果 = ["ab", " ab", " cd"]

输出: {
    (97, 98): 1,           # "ab" 出现1次
    (32, 97, 98): 1,       # " ab" 出现1次
    (32, 99, 100): 1       # " cd" 出现1次
}

解释:
- "ab" 的 UTF-8 字节为 [97, 98]
- " ab" 的 UTF-8 字节为 [32, 97, 98]
- " cd" 的 UTF-8 字节为 [32, 99, 100]
```

### 约束条件

- `0 <= num_special <= 1000`
- 文本长度 <= 10^6

### 函数签名

```python
def count_word_frequencies(text: str, num_special: int) -> dict[tuple[int, ...], int]:
    pass
```

---

## 3. 构建倒排索引与字节对频率表

**难度**: 中等

### 题目描述

给定一个词频字典，构建两个数据结构：
1. **字节对频率表**: 统计所有相邻字节对的总频率
2. **倒排索引**: 记录每个字节对出现在哪些词中

倒排索引的作用是在后续合并操作时，能够快速定位包含特定字节对的词，避免遍历所有词。

### 示例

```
输入:
freqs = {
    (1, 2, 3): 2,    # 词 [1,2,3] 出现2次
    (1, 2): 3        # 词 [1,2] 出现3次
}

输出:
pair_freqs = {
    (1, 2): 5,       # 对(1,2) 总频率 = 2 + 3 = 5
    (2, 3): 2        # 对(2,3) 总频率 = 2
}

pair_to_words = {
    (1, 2): {(1, 2, 3), (1, 2)},  # 包含(1,2)的词集合
    (2, 3): {(1, 2, 3)}            # 包含(2,3)的词集合
}

解释:
- 遍历每个词，提取所有相邻字节对
- 同时更新频率表和倒排索引
```

### 约束条件

- 词频字典大小 <= 10^6
- 每个词的长度 <= 100
- 频率值 >= 1

### 函数签名

```python
def build_index(freqs: dict[tuple[int, ...], int]) -> tuple[
    dict[tuple[int, int], int],                    # pair_freqs
    dict[tuple[int, int], set[tuple[int, ...]]]    # pair_to_words
]:
    pass
```

---

## 4. 最大堆与懒删除

**难度**: 中等

### 题目描述

设计一个支持动态频率更新的最大堆数据结构，用于高效获取频率最高的字节对。

要求：
1. **插入**: 将 (频率, 字节对) 插入堆
2. **获取最大**: 返回频率最高的字节对
3. **Tie-breaking**: 频率相同时，选择字节序列字典序**最大**的
4. **懒删除**: 频率会动态变化，获取时需验证并跳过过期条目

**关键难点**: Python `heapq` 是最小堆，需要技巧处理：
- 用负数模拟最大堆
- 频率相同时要选字典序最大的，需反转 bytes 比较

### 示例

```
输入操作序列:
push((1, 2), freq=100, bytes=(b'a', b'b'))
push((3, 4), freq=100, bytes=(b'c', b'd'))
push((5, 6), freq=50,  bytes=(b'e', b'f'))
# 外部更新: pair_freqs[(1, 2)] = 0
get_best()

输出: (3, 4)

解释:
- (1,2) 和 (3,4) 频率相同=100
- (3,4) 的 bytes (b'c',b'd') 字典序大于 (b'a',b'b')
- 但 (1,2) 实际频率已变为0，是过期条目
- 懒删除跳过 (1,2)，最终返回 (3,4)
```

### 约束条件

- 字节对数量 <= 10^6
- 频率值 <= 10^9

### 函数签名

```python
class _ReversedBytes:
    """反转 bytes 比较顺序，使最小堆选择字典序最大的"""
    __slots__ = ('b',)
    def __init__(self, b: bytes):
        self.b = b
    def __lt__(self, other):
        return self.b > other.b  # 反转比较

class PairHeap:
    def __init__(self, id_to_bytes: dict[int, bytes]):
        self.heap = []
        self.id_to_bytes = id_to_bytes

    def push(self, pair: tuple[int, int], freq: int) -> None:
        """插入堆，元素格式: (-freq, ReversedBytes_a, ReversedBytes_b, pair)"""
        pass

    def get_best(self, pair_freqs: dict) -> tuple[int, int] | None:
        """获取最大频率对，懒删除过期条目"""
        pass
```

---

## 5. 字节对合并（倒排索引优化）

**难度**: 中等

### 题目描述

给定词频字典、倒排索引和字节对频率表，将所有词中的字节对 `(a, b)` 合并为 `new_id`。

**优化要求**: 使用倒排索引，只处理包含目标字节对的词，而非遍历所有词。

合并规则：
1. 通过倒排索引获取包含 `(a, b)` 的词集合
2. 对于每个相关词：
   - 从倒排索引移除该词的所有字节对映射
   - 更新字节对频率（减旧加新）
   - 执行合并，生成新词
   - 将新词的字节对加入倒排索引
3. 将受影响的字节对重新推入堆

### 示例

```
输入:
freqs = {(1, 2, 3): 2, (1, 2, 1, 2): 1}
pair_to_words = {
    (1, 2): {(1, 2, 3), (1, 2, 1, 2)},
    (2, 3): {(1, 2, 3)},
    (2, 1): {(1, 2, 1, 2)}
}
a = 1, b = 2, new_id = 100

输出:
freqs = {(100, 3): 2, (100, 100): 1}
pair_to_words = {
    (100, 3): {(100, 3)},
    (100, 100): {(100, 100)}
}

解释:
- 只处理 pair_to_words[(1,2)] 中的 2 个词，而非遍历全部词
- 复杂度从 O(N) 降到 O(K)，K=2
```

### 约束条件

- 词频字典大小 <= 10^6
- `0 <= a, b, new_id <= 10^6`

### 函数签名

```python
def merge(
    freqs: dict[tuple[int, ...], int],
    pair_freqs: dict[tuple[int, int], int],
    pair_to_words: dict[tuple[int, int], set[tuple[int, ...]]],
    a: int,
    b: int,
    new_id: int,
) -> set[tuple[int, int]]:  # 返回受影响的字节对集合
    pass
```

---

## 6. BPE 词表构建（优化版）

**难度**: 困难

### 题目描述

实现 Byte Pair Encoding (BPE) 算法来构建词表，要求使用以下优化技术：

1. **最大堆**: O(1) 获取最高频字节对（替代 O(P) 的线性扫描）
2. **倒排索引**: O(K) 只处理相关词（替代 O(N) 遍历所有词）
3. **懒删除**: 高效处理频率动态变化

### 优化算法步骤

```
1. 初始化词表: 特殊 tokens + 256 个基础字节
2. 预分词: 并行处理文件，统计词频
3. 构建索引:
   - pair_freqs: 字节对 -> 频率
   - pair_to_words: 字节对 -> 词集合（倒排索引）
   - heap: 最大堆
4. 迭代合并:
   while vocab_size 未达到目标:
       best = heap.get_best()          # O(1) 摊销
       merge(best)                      # O(K × L), K << N
       更新索引和堆
```

### 示例

```
输入:
corpus = "low low low lower newest widest"
vocab_size = 262

输出:
vocab = {0: b'l', 1: b'o', ..., 256: b'lo', 257: b'low', ...}
merges = [(b'l', b'o'), (b'lo', b'w'), ...]

解释:
- 使用堆 O(1) 获取最高频对，而非 O(P) 遍历
- 使用倒排索引只处理包含目标对的词
```

### 约束条件

- `257 <= vocab_size <= 100000`
- 特殊 token 数量 <= 100
- 语料大小 <= 10^9 字节

### 函数签名

```python
class BPETrainer:
    """封装堆和倒排索引的 BPE 训练器"""
    def __init__(self, freqs: dict, id_to_bytes: dict): ...
    def get_best_pair(self) -> tuple[int, int] | None: ...
    def merge(self, a: int, b: int, new_id: int) -> None: ...

def train_bpe(
    input_path: str,
    vocab_size: int,
    special_tokens: list[str],
) -> tuple[dict[int, bytes], list[tuple[bytes, bytes]]]:
    pass
```

---

## 7. 并行预分词

**难度**: 困难

### 题目描述

设计一个并行预分词系统，将大文件分成多个块并行处理，最后合并结果。

要求：
1. 将文件分成 N 个块（N = CPU 核心数）
2. 每个块的边界必须对齐到特殊 token
3. 并行处理每个块，统计词频
4. 合并所有块的词频统计结果

### 示例

```
输入:
file_content = "hello world<|endoftext|>foo bar<|endoftext|>test"
special_tokens = ["<|endoftext|>"]
num_workers = 2

处理过程:
- 块1: "hello world<|endoftext|>" -> {"hello": 1, " world": 1}
- 块2: "foo bar<|endoftext|>test" -> {"foo": 1, " bar": 1, "test": 1}

输出: {"hello": 1, " world": 1, "foo": 1, " bar": 1, "test": 1}
（实际输出为字节元组形式）
```

### 约束条件

- 文件大小 <= 10^9 字节
- 特殊 token 数量 <= 100
- 需要考虑多进程并行效率

### 函数签名

```python
def pre_tokenize(
    input_path: str,
    special_tokens: list[str],
    num_special: int
) -> dict[tuple[int, ...], int]:
    pass
```

---

## 算法复杂度总结

| 题目 | 原始复杂度 | 优化后复杂度 |
|------|-----------|-------------|
| 1. 文件分块边界查找 | O(C × 4096) | - |
| 2. 词频统计 | O(n) | - |
| 3. 构建倒排索引 | O(N × L) | - |
| 4. 获取最大频率对 | **O(P)** | **O(1) 摊销** |
| 5. 字节对合并 | **O(N × L)** | **O(K × L)** |
| 6. BPE 词表构建 | **O(V × (P + N × L))** | **O(V × log P + Σ K × L)** |
| 7. 并行预分词 | O(n / W) | - |

### 符号说明

| 符号 | 含义 |
|------|------|
| n | 文本总长度 |
| N | 唯一词数量 |
| L | 平均词长度 |
| P | 字节对数量 |
| V | 合并次数 (vocab_size - 256) |
| K | 包含特定字节对的词数量 (**K << N**) |
| C | 分块数量 |
| W | 并行进程数 |

### 优化效果

| 优化技术 | 作用 | 加速比 |
|---------|------|-------|
| 最大堆 | 避免线性扫描找最大值 | O(P) → O(1) |
| 倒排索引 | 避免遍历无关词 | O(N) → O(K) |
| 懒删除 | 高效处理频率变化 | 避免堆重建 |
| _ReversedBytes | 保证 tie-breaking 顺序正确 | - |
