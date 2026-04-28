#!/usr/bin/env python3
"""
CLIP Zero-Shot Classification Sidecar
为 Rust 主进程提供 CLIP 图像分类服务。

通过 stdin/stdout 进行通信：
- 输入: 图像文件路径 (每行一个)
- 输出: JSON 分类结果
- 特殊命令: HEALTHCHECK

依赖: torch, torchvision, open_clip_torch
"""

import sys
import json
import os
from pathlib import Path

try:
    import torch
    from PIL import Image
    import open_clip
except ImportError as e:
    print(f"ERROR: Missing dependency: {e}", file=sys.stderr)
    print("Please run: pip install torch torchvision open_clip_torch Pillow", file=sys.stderr)
    sys.exit(1)


def parse_args():
    """解析命令行参数"""
    args = sys.argv[1:]
    model_name = "ViT-B/32"
    categories = []

    i = 0
    while i < len(args):
        if args[i] == "--model" and i + 1 < len(args):
            model_name = args[i + 1]
            i += 2
        elif args[i] == "--categories" and i + 1 < len(args):
            categories = json.loads(args[i + 1])
            i += 2
        else:
            i += 1

    return model_name, categories


def load_model(model_name: str, categories: list):
    """加载 CLIP 模型"""
    print(f"Loading CLIP model: {model_name}", file=sys.stderr)

    model, _, preprocess = open_clip.create_model_and_transforms(model_name)
    tokenizer = open_clip.get_tokenizer(model_name)

    text_prompts = [f"a photo of {cat}" for cat in categories]
    text_tokens = tokenizer(text_prompts)

    model.eval()

    return model, preprocess, tokenizer, text_tokens


def classify_image(
    image_path: str, model, preprocess, text_tokens
) -> dict:
    """对单张图像进行分类"""
    try:
        image = Image.open(image_path).convert("RGB")
        image_input = preprocess(image).unsqueeze(0)
    except Exception as e:
        return {"error": f"Image load failed: {str(e)}"}

    with torch.no_grad():
        image_features = model.encode_image(image_input)
        image_features = image_features / image_features.norm(dim=-1, keepdim=True)

        text_features = model.encode_text(text_tokens)
        text_features = text_features / text_features.norm(dim=-1, keepdim=True)

        similarities = (image_features @ text_features.T).softmax(dim=-1)
        probabilities = similarities[0].tolist()

    top_indices = sorted(
        range(len(probabilities)),
        key=lambda i: probabilities[i],
        reverse=True,
    )

    categories = [f"a photo of {cat}" for cat in get_categories_from_args()]
    top_categories = []
    for idx in top_indices[:5]:
        cat_name = categories[idx].replace("a photo of ", "")
        top_categories.append([cat_name, probabilities[idx]])

    result = {
        "category": top_categories[0][0] if top_categories else "other",
        "confidence": top_categories[0][1] if top_categories else 0.0,
        "top_categories": top_categories,
    }

    return result


def get_categories_from_args() -> list:
    """从命令行参数获取类别列表"""
    args = sys.argv[1:]
    i = 0
    while i < len(args):
        if args[i] == "--categories" and i + 1 < len(args):
            return json.loads(args[i + 1])
        i += 1
    return ["landscape", "person", "object", "animal", "architecture", "document", "other"]


def main():
    model_name, categories = parse_args()

    if not categories:
        categories = ["landscape", "person", "object", "animal", "architecture", "document", "other"]

    model, preprocess, tokenizer, text_tokens = load_model(model_name, categories)
    print(f"CLIP model loaded successfully", file=sys.stderr)

    sys.stdout.flush()

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        if line == "HEALTHCHECK":
            print("OK", flush=True)
            continue

        result = classify_image(line, model, preprocess, text_tokens)

        if "error" in result:
            print(f"ERROR: {result['error']}", flush=True)
        else:
            print(json.dumps(result), flush=True)


if __name__ == "__main__":
    main()
