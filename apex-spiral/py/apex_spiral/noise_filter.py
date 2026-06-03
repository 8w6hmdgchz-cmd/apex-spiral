#!/usr/bin/env python3
"""APEX V11 noise_filter — 压噪声 (H 0.45→0.35)
检测并删除空话/套话/重复句.
"""
import re

NOISE_PATTERNS = [
    r'^(应该|或许|可能|建议|可以考虑|也许).{0,20}(做|试试|看看|弄)',
    r'^(好的|收到|明白|了解|知道了|嗯|哦|啊)[。!！.\\s]*$',
    r'本.{0,5}可以帮助|如果您需要|随时.{0,5}问',
    r'我.{0,5}理解.{0,20}需求',
    r'总.{0,5}而言|综合.{0,5}来看|综上所述',
]

def is_noise(line: str) -> bool:
    s = line.strip()
    if not s: return True
    if len(s) < 3: return True
    for p in NOISE_PATTERNS:
        if re.search(p, s): return True
    return False

def filter_output(text: str) -> str:
    """返回去噪后的文本 + 噪声率."""
    lines = text.split('\n')
    kept = [l for l in lines if not is_noise(l)]
    noise_rate = 1 - len(kept)/max(len(lines),1)
    return '\n'.join(kept), noise_rate

if __name__ == "__main__":
    sample = """好的，收到。
我理解了您的需求。
也许我们可以试试这样做。
ΔG=39.41 是当前基线。
建议您可以考虑使用 rtk。
没问题，随时问我。"""
    out, rate = filter_output(sample)
    print(f"noise_rate={rate:.2%}")
    print("kept:")
    print(out)
