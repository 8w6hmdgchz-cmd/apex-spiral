# Apex-Spiral Evolver Docker Image
FROM python:3.11-slim

LABEL maintainer="node_cfd285ff67c1"
LABEL description="AI Self-Evolver based on Apex Formula"

# 安装基础工具
RUN apt-get update && apt-get install -y \
    curl \
    git \
    openssh-client \
    && rm -rf /var/lib/apt/lists/*

# 工作目录
WORKDIR /app

# 复制核心脚本
COPY apex-iterate.sh /app/
COPY evolver-hub-sync.sh /app/
COPY a2a-resource-*.sh /app/

# 设置执行权限
RUN chmod +x /app/*.sh

# 环境变量
ENV PYTHONUNBUFFERED=1
ENV EVO_NODE_ID=node_cfd285ff67c1
ENV GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=no"

# 健康检查
HEALTHCHECK --interval=15m --timeout=10s --start-period=5m --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# 默认命令：运行迭代
CMD ["/app/apex-iterate.sh"]
