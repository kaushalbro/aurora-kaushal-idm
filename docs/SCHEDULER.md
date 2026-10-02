# AURORA Scheduler Mathematics & Dynamic Schedulers

This document provides the formal mathematical definitions and algorithmic specifications for all four scheduling strategies implemented in the AURORA engine, with special emphasis on the **AURORA Expected Completion Time (ECT)** scheduler.

---

## 1. Problem Formulation & Definitions

Let a file download task of total length $L \in \mathbb{N}$ bytes be represented on a continuous or discrete interval $[0, L - 1]$.

At any discrete time $t$, let:
- $\mathcal{S}(t) = \{S_1, S_2, \dots, S_m\}$ be the active partitioning of segments, where segment $i$ covers range $[a_i, b_i]$ with total bytes $L_i = b_i - a_i + 1$.
- $D_i(t) \in [0, L_i]$ be the bytes successfully downloaded and committed to disk for segment $i$.
- $R_i(t) = L_i - D_i(t)$ be the remaining un-downloaded bytes for segment $i$.
- $\mathcal{W}(t) = \{W_1, W_2, \dots, W_k\}$ be the set of active worker connections.
- $r_j(t) > 0$ be the smoothed download rate (bytes/sec) of worker $W_j$ estimated via EWMA:
  $$r_j(t) = \alpha \cdot r_{j,\text{instant}}(t) + (1 - \alpha) \cdot r_j(t - \Delta t)$$
  where $\alpha = 1 - e^{-\Delta t / \tau}$ with default time constant $\tau = 1.5\,\text{s}$.

---

## 2. Scheduler Algorithms

### Scheduler A — Single Stream (Baseline)
- Concurrency: $|\mathcal{W}| = 1$.
- Partition: Single continuous range $[0, L - 1]$.
- No work stealing or dynamic splitting.

### Scheduler B — Fixed Segmentation
- Concurrency: $|\mathcal{W}| = k$.
- Static partitioning into $k$ equal slices at initialization:
  $$S_i = \left[ (i - 1) \cdot \left\lfloor \frac{L}{k} \right\rfloor, \; i \cdot \left\lfloor \frac{L}{k} \right\rfloor - 1 \right] \quad (1 \le i < k)$$
  $$S_k = \left[ (k - 1) \cdot \left\lfloor \frac{L}{k} \right\rfloor, \; L - 1 \right]$$
- No work stealing occurs during execution. If worker $W_a$ is $5\times$ faster than worker $W_b$, $W_a$ finishes early and sits idle while $W_b$ becomes a straggler.

### Scheduler C — Largest-Segment Splitting (IDM-Style Baseline)
- When any worker $W_{\text{idle}}$ finishes its assigned range and becomes idle:
  1. Find the segment with the largest remaining byte count:
     $$S^* = \operatorname{argmax}_{S_i \in \mathcal{S}_{\text{active}}} R_i(t)$$
  2. If $R^*(t) \ge 2 \cdot S_{\text{min}}$ (where $S_{\text{min}}$ is minimum split threshold, default $1\,\text{MB}$):
     Split remaining bytes 50/50:
     $$R_{\text{existing}} = \left\lfloor \frac{R^*}{2} \right\rfloor, \quad R_{\text{stolen}} = R^* - R_{\text{existing}}$$
     Existing worker continues with $[a^* + D^*, \; a^* + D^* + R_{\text{existing}} - 1]$.
     Idle worker $W_{\text{idle}}$ receives stolen range $[a^* + D^* + R_{\text{existing}}, \; b^*]$.

---

## 3. Scheduler D — AURORA Expected Completion Time (ECT)

The core insight of AURORA-ECT is that **the largest remaining range is not necessarily the last to complete** when worker transfer rates are non-uniform.

### 3.1 Expected Completion Time Formulation
For each active segment $i$ handled by worker with estimated speed $r_i$, we calculate:
$$\text{ECT}_i(t) = \frac{R_i(t)}{\max(r_i(t), r_{\text{min}})} + T_{\text{setup}}$$
where $T_{\text{setup}}$ represents connection setup, TLS handshake, and HTTP range negotiation latency (default $\sim 200\,\text{ms}$).

### 3.2 Straggler Target Selection
Instead of selecting $\operatorname{argmax} R_i$, AURORA selects the worst predicted completion bottleneck:
$$S^* = \operatorname{argmax}_{S_i \in \mathcal{S}_{\text{active}}} \text{ECT}_i(t)$$

### 3.3 Rate-Proportional Work Stealing
When an idle worker $W_b$ (with rate $r_b$) steals work from straggler worker $W_a$ (with rate $r_a$), splitting 50/50 creates another imbalance if $r_a \neq r_b$.

To equalize completion times:
$$\frac{R_a'}{r_a} \approx \frac{R_b'}{r_b} \implies R_a' + R_b' = R^*$$
Solving for portions:
$$R_a' = R^* \cdot \frac{r_a}{r_a + r_b}, \qquad R_b' = R^* \cdot \frac{r_b}{r_a + r_b}$$

**Example**:
- Remaining straggler data $R^* = 1\,\text{GB}$.
- Existing worker $W_a$ running at $25\,\text{MB/s}$.
- Idle worker $W_b$ running at $75\,\text{MB/s}$.
- Portions assigned:
  - $W_a \gets 1\,\text{GB} \cdot \frac{25}{100} = 250\,\text{MB}$ (Predicted time: $10\,\text{s}$)
  - $W_b \gets 1\,\text{GB} \cdot \frac{75}{100} = 750\,\text{MB}$ (Predicted time: $10\,\text{s}$)
- Under 50/50 splitting, $W_a$ would take $20\,\text{s}$ while $W_b$ finished in $6.67\,\text{s}$, creating an artificial $13.3\,\text{s}$ straggler delay.

---

## 4. Adaptive Segment Duration Sizing

Segment sizes are dynamically clamped based on worker speed and target duration $\Delta t_{\text{target}}$ (default $4\,\text{s}$):
$$S_{\text{size}} = \operatorname{clamp}\left(r_j(t) \cdot \Delta t_{\text{target}}, \; S_{\text{min}}, \; S_{\text{max}}\right)$$

This ensures that high-speed fiber connections receive larger slices (reducing connection churn) while slower connections receive smaller slices (enabling nimble rebalancing).

---

## 5. Adaptive Concurrency Control

AURORA continuously evaluates marginal throughput gains before expanding worker counts ($k \to k + 1$):
$$\text{Gain} = \frac{\text{Throughput}_{k+1} - \text{Throughput}_k}{\text{Throughput}_k}$$

If $\text{Gain} < \text{Threshold}_{\text{marginal}}$ (default $5\%$) or if $\text{RTT Inflation} = \frac{\text{RTT}_{\text{curr}} - \text{RTT}_{\text{min}}}{\text{RTT}_{\text{min}}} > 50\%$, concurrency is stepped down to prevent bufferbloat and server rate limiting.
