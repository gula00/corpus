# Tokenizer Training Answers

## 2.6 TinyStories Tokenizer Training

**(a) Training time, memory, longest token:**
Training took ~6 minutes (354s with parallelization, 939s serial) and used ~48-73 MB RAM—well within the 30 min / 30GB limits. The longest tokens are " accomplishment", " disappointment", and " responsibility" (15 bytes each), which make sense as they are common words in children's moral stories and GPT-2 style tokenization includes the leading space.

**(b) Profiling - what takes the most time:**
Pre-tokenization (regex splitting into words) and BPE merging each take significant time. Pre-tokenization parallelizes well (11.4x speedup with 20 workers), but BPE merging is inherently sequential (only 1.08x speedup) because each merge depends on the previous state—this is the main bottleneck.

## 2.7 OpenWebText Tokenizer Training

**(a) Longest token:**
The longest readable token is " disproportionately" or " telecommunications" (19 bytes). The absolute longest token (64 bytes) is a garbled UTF-8 sequence `\xc3\x83\xc3\x82...` which represents repeated double-encoding artifacts from web scraping—this doesn't make semantic sense but appears frequently enough in noisy web text to be merged.

**(b) TinyStories vs OpenWebText comparison:**
The TinyStories tokenizer has simpler vocabulary dominated by children's story words (" granddaughter", " strawberries", " caterpillars"), while OWT has more complex/technical vocabulary (" telecommunications", " cryptocurrencies", " unconstitutional"). OWT also contains web-specific tokens and encoding artifacts that don't appear in the curated TinyStories dataset.

## 2.8 Tokenizer Analysis

**(a) Compression ratio on 10 sampled documents:**
TinyStories tokenizer on TinyStories achieves 4.14 bytes/token; OpenWebText tokenizer on OpenWebText achieves 4.30 bytes/token. Both tokenizers achieve good compression on their native datasets, with similar efficiency despite OWT having 3x larger vocabulary.

**(b) Cross-tokenization (TS tokenizer on OWT):**
TinyStories tokenizer on OpenWebText achieves only 3.44 bytes/token (worse than OWT's 4.30). The TS tokenizer lacks web-specific vocabulary like "supermarket" (split into "super"+"m"+"ark"+"et") and technical terms, resulting in more fragmented tokens and worse compression.

**(c) Throughput and Pile estimation:**
TinyStories tokenizer throughput is ~9 MB/s, OWT tokenizer is ~2.5 MB/s (slower due to larger vocab/merges). Encoding the Pile (825GB) would take approximately 25-100 hours (1-4 days) depending on the tokenizer—parallelization across documents would significantly reduce this.

**(d) Why uint16 for token IDs:**
uint16 (max value 65535) is appropriate because both vocabulary sizes (10k and 32k) fit within this range, using only 2 bytes per token instead of 4 bytes for uint32, halving storage requirements (e.g., TinyStories train: 1.08GB instead of 2.16GB).

---

# 3. Transformer Language Model Architecture

## 3.1 Resource Accounting (transformer_accounting)

**Model Configuration (GPT-2 XL):**
- Vocabulary size: 50,257
- Context length (T): 1,024
- Layers (L): 48
- Model dimension (d_model): 1,600
- Attention heads: 25
- FFN dimension (d_ff): 6,400

### (a) Parameter Count and Memory

**Parameters breakdown:**

| Component | Formula | Count |
|-----------|---------|-------|
| Token Embedding | vocab × d_model | 50,257 × 1,600 = 80,411,200 |
| RMSNorm (per block, ×2) | 2 × d_model × L | 2 × 1,600 × 48 = 153,600 |
| Attention Q,K,V,O | 4 × d_model² × L | 4 × 2,560,000 × 48 = 491,520,000 |
| SwiGLU FFN (W1,W2,W3) | 3 × d_model × d_ff × L | 3 × 10,240,000 × 48 = 1,474,560,000 |
| Final RMSNorm | d_model | 1,600 |
| LM Head | Tied with embedding | 0 |

**Total Parameters: 2,046,646,400 ≈ 2.05B**

**Memory (float32):** 2.05B × 4 bytes = **8.19 GB**

### (b) Matrix Multiplies and FLOPs (Forward Pass)

For batch size B=1, sequence length T=1,024:

**Per Transformer Block:**

| Operation | Formula | FLOPs |
|-----------|---------|-------|
| Q projection | 2 × T × d² | 4.19B |
| K projection | 2 × T × d² | 4.19B |
| V projection | 2 × T × d² | 4.19B |
| QK^T attention | 2 × T² × d | 3.36B |
| Softmax × V | 2 × T² × d | 3.36B |
| Output projection | 2 × T × d² | 4.19B |
| FFN W1 (gate) | 2 × T × d × d_ff | 20.97B |
| FFN W2 (up) | 2 × T × d × d_ff | 20.97B |
| FFN W3 (down) | 2 × T × d_ff × d | 20.97B |
| **Per block total** | | **90.6B** |

**All 48 blocks:** 48 × 90.6B = **4.35T**

**LM Head:** 2 × T × d × vocab = 2 × 1,024 × 1,600 × 50,257 = **164.7B**

**Total Forward Pass FLOPs: ~4.5T** (4.5 trillion FLOPs per sequence)

### (c) FLOPs Breakdown by Component

| Component | FLOPs | Percentage |
|-----------|-------|------------|
| FFN (SwiGLU) | 3.02T | 67% |
| Attention linear (Q,K,V,O) | 1.01T | 22% |
| Attention QK^T + softmax×V | 0.32T | 7% |
| LM Head | 0.17T | 4% |

**The FFN layers dominate** the compute, taking ~2/3 of total FLOPs. This is because d_ff = 4×d_model, so FFN has 3 matrices each of size d×4d, while attention has 4 matrices of size d×d.

### (d) Comparison Across GPT-2 Sizes

| Model | L | d_model | d_ff | Parameters | Forward FLOPs |
|-------|---|---------|------|------------|---------------|
| GPT-2 Small | 12 | 768 | 3,072 | 152M | 0.16T |
| GPT-2 Medium | 24 | 1,024 | 4,096 | 454M | 0.48T |
| GPT-2 Large | 36 | 1,280 | 5,120 | 1.01B | 1.10T |
| GPT-2 XL | 48 | 1,600 | 6,400 | 2.05B | 4.51T |

Scaling observations:
- Parameters scale roughly as d² × L (quadratic in d_model, linear in layers)
- FLOPs scale similarly, with slight superlinear effect from attention's T² term
- XL has ~13.5x more parameters than Small, and ~28x more FLOPs

### (e) Impact of Context Length 16,384

Increasing T from 1,024 to 16,384 (16× longer):

**Per Transformer Block at T=16,384:**

| Operation | T=1,024 | T=16,384 | Scale |
|-----------|---------|----------|-------|
| Attention linear (Q,K,V,O) | 16.8B | 268.4B | 16× (linear) |
| Attention QK^T + softmax×V | 6.7B | 1,717.0B | 256× (quadratic!) |
| FFN | 62.9B | 1,006.6B | 16× (linear) |
| **Per block total** | 90.6B | 2,992.0B | 33× |

**Total FLOPs at T=16,384:**
- 48 blocks: 143.6T
- LM Head: 2.6T
- **Total: ~146T** (vs 4.5T at T=1,024)

**FLOPs increase: ~33×** for 16× longer context

**New breakdown at T=16,384:**

| Component | FLOPs | Percentage |
|-----------|-------|------------|
| Attention QK^T + softmax×V | 82.4T | **56%** |
| FFN (SwiGLU) | 48.3T | 33% |
| Attention linear | 12.9T | 9% |
| LM Head | 2.6T | 2% |

**Key insight:** At long context lengths, the quadratic attention mechanism dominates compute instead of FFN. This is why efficient attention variants (FlashAttention, linear attention, sparse attention) are critical for long-context models.

---

# 4. Training a Transformer LM

## 4.1 Cross-Entropy Loss
Implementation: Use log-sum-exp trick for numerical stability:
```
ℓ = -log(softmax(o)[target]) = -o[target] + log(Σ exp(o))
```
Subtract max(o) before exp to avoid overflow.

## 4.2 Learning Rate Tuning (learning_rate_tuning)

**SGD with different learning rates (10 iterations on toy example):**

| Learning Rate | Behavior |
|---------------|----------|
| lr=1 (baseline) | Loss decays steadily |
| lr=1e1 (10) | Loss decays faster initially, may oscillate |
| lr=1e2 (100) | Loss oscillates wildly, likely unstable |
| lr=1e3 (1000) | Loss explodes/diverges immediately (NaN) |

**Observation:** Higher learning rates can accelerate initial progress but risk instability. Beyond a threshold, the optimizer "overshoots" the minimum, causing divergence. The optimal learning rate is typically just below the stability threshold.

## 4.3 AdamW Resource Accounting (adamwAccounting)

### (a) Peak Memory Breakdown

Let N = number of parameters, B = batch_size, T = context_length, d = d_model, L = num_layers, V = vocab_size, d_ff = 4d, h = num_heads.

**Parameters:**
- Token Embedding: V × d
- Per block: 2d (RMSNorms) + 4d² (attention) + 3d×d_ff (SwiGLU FFN)
- Final RMSNorm: d
- LM Head (if tied): 0; (if not tied): V × d

Total params: N ≈ V×d + L×(4d² + 12d²) + L×2d + d ≈ **V×d + 16Ld²**

**Memory components (float32 = 4 bytes):**

| Component | Formula | Memory |
|-----------|---------|--------|
| Parameters | 4N bytes | 4N |
| Gradients | 4N bytes | 4N |
| Optimizer state (m, v) | 2 × 4N = 8N bytes | 8N |
| **Subtotal (fixed)** | | **16N** |

**Activations (per forward pass):**

Per Transformer block:
- RMSNorm inputs: 2 × B×T×d
- Q, K, V projections: 3 × B×T×d
- Attention scores (QK^T): B×h×T×T
- Softmax output: B×h×T×T
- Weighted values: B×T×d
- Output projection: B×T×d
- FFN intermediates: 4 × B×T×d_ff (W1, W3, SiLU, gate×up)

Per block: ~B×T×(8d + 4d_ff) + 2B×T²×h

All L blocks + output:
**Activations ≈ 4 × L × B × T × (8d + 16d) + 4 × 2L × B × T² × h + 4 × B × T × V**

Simplified: **A ≈ 4BTL(24d + 2Th/L) + 4BTV ≈ 96BTLd + 8BT²h + 4BTV**

**Total Peak Memory:**
```
Memory = 16N + 96BTLd + 8BT²hL + 4BTV
```

### (b) GPT-2 XL Instantiation

**GPT-2 XL config:** V=50,257, T=1,024, L=48, d=1,600, h=25, d_ff=6,400

**Parameters:** N ≈ 2.05B (from Section 3)
- Fixed memory: 16N = 16 × 2.05B × 4 bytes = **32.8 GB**

**Activations (simplified estimate):**
- Per block: ~B × T × (8d + 4d_ff) = B × 1024 × (12,800 + 25,600) = 39.3M × B
- Attention: 2 × B × h × T² = 2 × B × 25 × 1024² = 52.4M × B
- Per block total: ~92M × B elements
- 48 blocks: 4.4B × B elements
- LM head: B × T × V = 51.5M × B

Total activations: ~4.5B × B elements → **18 GB × B** (float32)

**Memory formula:** Memory ≈ 32.8 + 18×B GB

**Maximum batch size for 80GB:**
```
32.8 + 18×B ≤ 80
B ≤ (80 - 32.8) / 18 ≈ 2.6
```
**Maximum batch_size ≈ 2** (conservative; actual may be ~4 with optimizations)

### (c) FLOPs for One AdamW Step

For each of N parameters, AdamW performs:
- m update: 2 ops (β₁m + (1-β₁)g)
- v update: 3 ops (β₂v + (1-β₂)g²)
- Bias correction + step: ~6 ops (sqrt, divide, multiply)
- Weight decay: 2 ops

**Total: ~13 FLOPs per parameter**

For GPT-2 XL: **13 × 2.05B ≈ 26.7 billion FLOPs** per optimizer step

(Note: This is negligible compared to forward/backward pass FLOPs)

### (d) Training Time Estimation

**Setup:**
- GPT-2 XL: N ≈ 2B params
- Steps: 400K
- Batch size: 1,024 sequences × 1,024 tokens = 1M tokens/step
- A100: 19.5 TFLOP/s (FP32), 50% MFU → 9.75 TFLOP/s effective

**FLOPs calculation:**
Using 6N FLOPs per token approximation (2N forward + 4N backward):
- FLOPs per token = 6 × 2B = 12B FLOPs
- FLOPs per step = 1M × 12B = **12 PFLOPs**
- Total FLOPs = 400K × 12P = **4.8 × 10²¹ FLOPs**

**Training time:**
```
Time = 4.8 × 10²¹ / (9.75 × 10¹²) = 4.9 × 10⁸ seconds ≈ 5,700 days ≈ 15.6 years
```

**This illustrates why:**
1. FP32 training is impractical for large models
2. Mixed precision (BF16/FP16) is essential: A100 delivers 312 TFLOP/s for TF32
3. With TF32 at 50% MFU (156 TFLOP/s): **~308 days ≈ 10 months**
4. Multi-GPU training is necessary for practical timelines

## 4.4 Learning Rate Schedule

Cosine annealing with warmup:
- **Warmup (t < Tw):** αt = (t/Tw) × αmax
- **Cosine decay (Tw ≤ t ≤ Tc):** αt = αmin + ½(1 + cos((t-Tw)/(Tc-Tw) × π))(αmax - αmin)
- **Post-annealing (t > Tc):** αt = αmin

## 4.5 Gradient Clipping

Clips gradient norm to maximum M:
```python
norm = sqrt(sum(grad² for all params))
if norm > M:
    scale = M / (norm + ε)  # ε = 1e-6
    for param in params:
        param.grad *= scale
```

---

## Raw Output: TinyStories BPE Training

```
Target vocab size: 10000
Special tokens: ['<|endoftext|>']
Initial memory: 25.32 MB
--------------------------------------------------
Pre-tokenize (parallel): start
  Split into 20 chunks using 20 workers
Pre-tokenize (parallel): finished in 54.06s
Init vocab: start
Init vocab: finished in 0.00s
Build pair counts: start
Build pair counts: finished in 0.05s
Merge: start
Merge: finished in 300.49s
Save merges: start
Save merges: finished in 0.00s
Save vocab: start
Save vocab: finished in 0.01s
Training completed in 354.63s
--------------------------------------------------
Training completed!
Time elapsed: 354.63 seconds (0.0985 hours)
Final memory: 72.99 MB
Memory increase: 47.67 MB
Vocab size: 10000
Merges performed: 9743
--------------------------------------------------
Longest token analysis:
  Token ID: 7160
  Bytes: b' accomplishment'
  Length: 15 bytes
  Decoded: ' accomplishment'
--------------------------------------------------

Performance comparison (serial vs parallel):
┌─────────┬──────────────────┬───────────────────┬────────┐
│  阶段   │ 串行 (streaming) │ 并行 (20 workers) │ 加速比 │
├─────────┼──────────────────┼───────────────────┼────────┤
│ 预分词  │ 615.82s          │ 54.06s            │ 11.4x  │
├─────────┼──────────────────┼───────────────────┼────────┤
│ BPE合并 │ 323.64s          │ 300.49s           │ 1.08x  │
├─────────┼──────────────────┼───────────────────┼────────┤
│ 总计    │ 939.54s          │ 354.63s           │ 2.65x  │
└─────────┴──────────────────┴───────────────────┴────────┘

Longest tokens (15 bytes):
┌──────────┬─────────────────┬──────────┐
│ Token ID │      Token      │  Length  │
├──────────┼─────────────────┼──────────┤
│ 7157     │  accomplishment │ 15 bytes │
├──────────┼─────────────────┼──────────┤
│ 9140     │  disappointment │ 15 bytes │
├──────────┼─────────────────┼──────────┤
│ 9376     │  responsibility │ 15 bytes │
└──────────┴─────────────────┴──────────┘
```

## Raw Output: Tokenizer Analysis (analyze_tokenizers.py)

```
======================================================================
TOKENIZER ANALYSIS
======================================================================

Loading tokenizers...
  TinyStories tokenizer: vocab=10000
  OpenWebText tokenizer: vocab=32000

======================================================================
Q3a: Compression ratios (10 sampled documents)
======================================================================

Sampling 10 documents from TinyStories...
  Sampled 10 documents, total 7253 bytes
  TinyStories tokenizer on TinyStories: 4.14 bytes/token

Sampling 10 documents from OpenWebText...
  Sampled 10 documents, total 91080 bytes
  OpenWebText tokenizer on OpenWebText: 4.30 bytes/token

======================================================================
Q3b: Cross-tokenization (TS tokenizer on OWT sample)
======================================================================
  TinyStories tokenizer on OpenWebText: 3.44 bytes/token
  OpenWebText tokenizer on OpenWebText: 4.30 bytes/token
  Ratio difference: TS tok is 1.25x the compression of OWT tok on OWT data

  Qualitative example (first 200 chars of first OWT doc):
    Text: If you buy your chicken from the supermarket, here are a few things about its life that might make y...
    TS tokenizer: 50 tokens
    OWT tokenizer: 47 tokens

    TS tokens (first 20): ['If', ' you', ' buy', ' your', ' chicken', ' from', ' the', ' super', 'm', 'ark', 'et', ',', ' here', ' are', ' a', ' few', ' things', ' about', ' its', ' life']
    OWT tokens (first 20): ['If', ' you', ' buy', ' your', ' chicken', ' from', ' the', ' supermarket', ',', ' here', ' are', ' a', ' few', ' things', ' about', ' its', ' life', ' that', ' might', ' make']

======================================================================
Q3c: Throughput estimation
======================================================================
  Test text size: 7262 bytes
  Time for 3 repetitions: 0.002s
  Throughput: 8967331 bytes/second (8.97 MB/s)
  Estimated time for Pile (825GB): 25.6 hours (1.1 days)
  OWT tokenizer throughput: 2549957 bytes/second (2.55 MB/s)

======================================================================
Q3d: Encoding datasets to uint16 numpy arrays
======================================================================
  TinyStories vocab size: 10000 (max id: 9999)
  OpenWebText vocab size: 32000 (max id: 31999)
  uint16 max value: 65535
  uint16 is appropriate because both vocab sizes (10k, 32k) < 65535 = 2^16 - 1

  Encoding TinyStories train...
    Already exists: out/tinystories-train-tokens.npy (541229574 tokens, 1082.5 MB)
    Already exists: out/tinystories-valid-tokens.npy (5465885 tokens, 10.9 MB)
    Already exists: out/owt-train-tokens.npy (2727121739 tokens, 5454.2 MB)
    Saved out/owt-valid-tokens.npy: 66401129 tokens, 132.8 MB, took 50.4s

======================================================================
TOKENIZER COMPARISON (TinyStories vs OpenWebText)
======================================================================
  TS longest token: b' accomplishment' (15 bytes)
  OWT longest token: b'\xc3\x83\xc3\x82...' (64 bytes, garbled UTF-8 encoding artifact)

  Tokens only in TS: 2681
    Examples (longest): [' congratulated', ' granddaughter', ' strawberries', ' marshmallows', ' caterpillars', ' veterinarian', ' imaginations', ' grandparent', ' stethoscope', ' marshmallow']

  Tokens only in OWT: 24681
    Examples (longest): [' disproportionately', ' telecommunications', ' environmentalists', ' responsibilities', ' unconstitutional', ' cryptocurrencies', ' counterterrorism', ' characterization']
```
