#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""W1 双盲对拍：SplitMix64 独立重实现（python 版）。

纪律（PIT-M-002）：期望值「先实测产出、后固化」。本文件与 Rust 实现
（render-spike/src/rng.rs）各自独立书写（不同语言、不互抄输出），两侧输出
逐位比对一致后才把前 8 值固化进 rng.rs 黄金测试。脚本输出即证据档
runs/w1_python_first8.log 的原始内容。

算法（步进式；派工单 §3 给定形态）：
    state = seed
    每取数:
        state = (state + 0x9E3779B97F4A7C15) mod 2^64
        z = state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) mod 2^64
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) mod 2^64
        out = z ^ (z >> 31)
"""

MASK64 = (1 << 64) - 1


def splitmix64_next(state: int) -> "tuple[int, int]":
    """返回 (新状态, 输出)。"""
    state = (state + 0x9E3779B97F4A7C15) & MASK64
    z = state
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
    out = z ^ (z >> 31)
    return state, out


def first8(seed: int):
    state = seed
    out = []
    for _ in range(8):
        state, v = splitmix64_next(state)
        out.append(v)
    return out


if __name__ == "__main__":
    for i, v in enumerate(first8(42)):
        print(f"python[{i}] = {v}")
