#!/usr/bin/env python3
"""
MiniMax 视频生成工具
用法: python3 video_generator.py --prompt "描述" [--duration 6] [--resolution 1080P]
"""

import os
import time
import requests
import argparse
import json
from pathlib import Path

# API 配置
API_KEY = os.environ.get("MINIMAX_API_KEY", "sk-cp-…OnkA")
API_HOST = "https://api.minimaxi.com"
HEADERS = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

# 可用模型
MODELS = {
    "hailuo-2.3": "MiniMax-Hailuo-2.3",
    "hailuo-02": "MiniMax-Hailuo-02",
    "t2v-01": "T2V-01",
    "t2v-01-director": "T2V-01-Director"
}

# 可用分辨率
RESOLUTIONS = ["720P", "768P", "1080P"]

# 可用时长
DURATIONS = {
    "MiniMax-Hailuo-2.3": {"768P": [6, 10], "1080P": [6]},
    "MiniMax-Hailuo-02": {"768P": [6, 10], "1080P": [6]},
    "T2V-01": {"720P": [6], "768P": [6], "1080P": [6]},
    "T2V-01-Director": {"720P": [6], "768P": [6], "1080P": [6]}
}

def create_video_task(model: str, prompt: str, duration: int = 6, 
                      resolution: str = "768P", prompt_optimizer: bool = True) -> str:
    """创建视频生成任务"""
    url = f"{API_HOST}/v1/video_generation"
    
    payload = {
        "model": model,
        "prompt": prompt,
        "duration": duration,
        "resolution": resolution,
        "prompt_optimizer": prompt_optimizer
    }
    
    response = requests.post(url, headers=HEADERS, json=payload, timeout=30)
    response.raise_for_status()
    
    data = response.json()
    if data.get("base_resp", {}).get("status_code") != 0:
        error_msg = data.get("base_resp", {}).get("status_msg", "未知错误")
        raise Exception(f"创建任务失败: {error_msg}")
    
    task_id = data.get("task_id")
    print(f"✅ 任务已创建: {task_id}")
    return task_id


def query_task_status(task_id: str) -> dict:
    """查询任务状态"""
    url = f"{API_HOST}/v1/query/video_generation"
    params = {"task_id": task_id}
    
    response = requests.get(url, headers=HEADERS, params=params, timeout=30)
    response.raise_for_status()
    
    return response.json()


def wait_for_completion(task_id: str, poll_interval: int = 10, max_wait: int = 600) -> str:
    """等待任务完成，返回 file_id"""
    start_time = time.time()
    
    while True:
        elapsed = time.time() - start_time
        if elapsed > max_wait:
            raise TimeoutError(f"等待超时 ({max_wait}s)")
        
        result = query_task_status(task_id)
        status = result.get("status", "processing")
        
        print(f"[{int(elapsed)}s] 状态: {status}")
        
        if status == "Success":
            file_id = result.get("file_id")
            print(f"✅ 生成成功! file_id: {file_id}")
            return file_id
        elif status == "Fail":
            error_msg = result.get("base_resp", {}).get("status_msg", "未知错误")
            raise Exception(f"生成失败: {error_msg}")
        
        time.sleep(poll_interval)


def download_video(file_id: str, output_path: str) -> str:
    """下载视频文件"""
    # 获取下载链接
    url = f"{API_HOST}/v1/files/retrieve"
    params = {"file_id": file_id}
    
    response = requests.get(url, headers=HEADERS, params=params, timeout=30)
    response.raise_for_status()
    
    data = response.json()
    download_url = data.get("file", {}).get("download_url")
    
    if not download_url:
        raise Exception("无法获取下载链接")
    
    # 下载文件
    print(f"📥 下载视频: {output_path}")
    response = requests.get(download_url, timeout=120, stream=True)
    response.raise_for_status()
    
    with open(output_path, "wb") as f:
        for chunk in response.iter_content(chunk_size=8192):
            f.write(chunk)
    
    print(f"✅ 下载完成: {output_path}")
    return output_path


def generate_video(prompt: str, duration: int = 6, resolution: str = "768P",
                  model: str = "MiniMax-Hailuo-2.3", output_dir: str = ".",
                  prompt_optimizer: bool = True) -> str:
    """完整视频生成流程"""
    print(f"🎬 开始生成视频")
    print(f"   模型: {model}")
    print(f"   描述: {prompt}")
    print(f"   时长: {duration}s")
    print(f"   分辨率: {resolution}")
    print()
    
    # 1. 创建任务
    task_id = create_video_task(model, prompt, duration, resolution, prompt_optimizer)
    
    # 2. 等待完成
    file_id = wait_for_completion(task_id)
    
    # 3. 生成输出文件名
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    safe_name = "".join(c for c in prompt[:20] if c.isalnum() or c in (' ', '-', '_')).strip()
    output_path = os.path.join(output_dir, f"video_{timestamp}_{safe_name}.mp4")
    
    # 4. 下载
    download_video(file_id, output_path)
    
    return output_path


def concatenate_videos(video_list: list, output_path: str) -> str:
    """使用 ffmpeg 拼接多个视频"""
    print(f"🎞️ 拼接 {len(video_list)} 个视频...")
    
    # 创建临时文件列表
    list_file = os.path.join(os.path.dirname(output_path), "concat_list.txt")
    with open(list_file, "w") as f:
        for video in video_list:
            f.write(f"file '{video}'\n")
    
    # 执行拼接
    cmd = f'ffmpeg -y -f concat -safe 0 -i "{list_file}" -c copy "{output_path}"'
    os.system(cmd)
    
    # 清理临时文件
    os.remove(list_file)
    
    print(f"✅ 拼接完成: {output_path}")
    return output_path


def main():
    parser = argparse.ArgumentParser(description="MiniMax 视频生成工具")
    parser.add_argument("--prompt", "-p", required=True, help="视频描述")
    parser.add_argument("--duration", "-d", type=int, default=6, help="视频时长 (秒)")
    parser.add_argument("--resolution", "-r", default="768P", 
                       choices=RESOLUTIONS, help="视频分辨率")
    parser.add_argument("--model", "-m", default="MiniMax-Hailuo-2.3",
                       choices=list(MODELS.values()), help="视频模型")
    parser.add_argument("--output", "-o", default=".", help="输出目录")
    parser.add_argument("--no-optimizer", action="store_true", help="禁用 prompt 优化")
    
    args = parser.parse_args()
    
    # 确保输出目录存在
    os.makedirs(args.output, exist_ok=True)
    
    try:
        output_path = generate_video(
            prompt=args.prompt,
            duration=args.duration,
            resolution=args.resolution,
            model=args.model,
            output_dir=args.output,
            prompt_optimizer=not args.no_optimizer
        )
        print(f"\n🎉 视频生成成功: {output_path}")
    except Exception as e:
        print(f"\n❌ 错误: {e}")
        return 1
    
    return 0


if __name__ == "__main__":
    exit(main())
