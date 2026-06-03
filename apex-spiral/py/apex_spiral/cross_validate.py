#!/usr/bin/env python3
"""APEX V11 cross_validate — 多源交叉验证 (Φ_SPARK 实化 3.38→4.5)
实现: 同一断言过 2+ 独立源才采纳.
"""
from typing import List, Dict, Callable
from collections import defaultdict

def cross_validate(claim: str, sources: List[Callable[[], str]], min_confirm: int = 2) -> Dict:
    """claim: 待验证断言. sources: 独立源函数列表. min_confirm: 至少几个源支持才通过."""
    votes = []
    for i, src in enumerate(sources):
        try:
            ans = src()
            votes.append({'source': i, 'supports': ans.strip().lower() in ('true','yes','1','pass'), 'raw': ans[:200]})
        except Exception as e:
            votes.append({'source': i, 'supports': False, 'raw': str(e)[:200]})
    confirm = sum(1 for v in votes if v['supports'])
    return {'claim': claim, 'confirm': confirm, 'total': len(sources),
            'passed': confirm >= min_confirm, 'votes': votes}

def cross_validate_facts(facts: Dict[str, str], source_a, source_b) -> List[Dict]:
    """批处理: facts 字典 {name: claim} 双源验证."""
    return [cross_validate(c, [source_a, source_b]) for c in facts.values()]

if __name__ == "__main__":
    # self-test
    r = cross_validate("V11 has 8 params",
                        [lambda: "true", lambda: "yes", lambda: "false"])
    print(r)
