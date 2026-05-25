#!/bin/bash
# 修复 fetcher 脚本中的路径空格问题
cd /Users/liHongxin/.openclaw/workspace
sed -i '' 's/pending\. list/pending.list/g; s/absorbed\. list/absorbed.list/g; s/failed\. list/failed.list/g; s/fetcher\. log/fetcher.log/g' a2a-resource-fetcher.sh
echo "修复完成"
cat a2a-resource-fetcher.sh | head -15