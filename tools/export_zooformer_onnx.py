#!/usr/bin/env python3
"""Export Python-old ZooFormer checkpoints to ONNX for the Rust server.

Examples:
    python tools/export_zooformer_onnx.py \
        --checkpoint ../Python-old/checkpoints/zooformer_supervised_replay_v3.pt \
        --output models/zooformer.onnx
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export Zodiac ZooFormer checkpoint (.pt) to ONNX."
    )
    parser.add_argument(
        "--checkpoint",
        required=True,
        help="Path to the .pt checkpoint file.",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Path to write the .onnx model.",
    )
    parser.add_argument(
        "--python-old",
        default="../Python-old",
        help="Path to the Python-old project root.",
    )
    parser.add_argument(
        "--opset",
        type=int,
        default=18,
        help="ONNX opset version. Default: 18.",
    )
    parser.add_argument(
        "--history-len",
        type=int,
        default=16,
        help="History length used by the model. Default: 16.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    checkpoint_path = Path(args.checkpoint).expanduser().resolve()
    output_path = Path(args.output).expanduser().resolve()
    python_old_root = Path(args.python_old).expanduser().resolve()

    if not checkpoint_path.exists():
        print(f"checkpoint not found: {checkpoint_path}", file=sys.stderr)
        return 1

    if not python_old_root.exists():
        print(f"Python-old root not found: {python_old_root}", file=sys.stderr)
        return 1

    sys.path.insert(0, str(python_old_root))

    try:
        import torch
        from nn.model import ZooFormer, ZooFormerConfig
        from nn.state_encoder import (
            CELL_FEATURE_DIM,
            GLOBAL_FEATURE_DIM,
            HISTORY_FEATURE_DIM,
        )
    except Exception as exc:
        print(f"failed to import Python-old model code: {exc}", file=sys.stderr)
        return 1

    device = torch.device("cpu")
    checkpoint = torch.load(checkpoint_path, map_location=device)
    config = ZooFormerConfig(**checkpoint.get("config", {}))
    model = ZooFormer(config).to(device)
    model.load_state_dict(checkpoint["model_state_dict"])
    model.eval()

    batch_size = 1
    history_len = args.history_len
    num_actions = 600

    cell_features = torch.zeros(
        (batch_size, 24, CELL_FEATURE_DIM), dtype=torch.float32, device=device
    )
    global_features = torch.zeros(
        (batch_size, GLOBAL_FEATURE_DIM), dtype=torch.float32, device=device
    )
    history_features = torch.zeros(
        (batch_size, history_len, HISTORY_FEATURE_DIM),
        dtype=torch.float32,
        device=device,
    )
    history_mask = torch.zeros(
        (batch_size, history_len), dtype=torch.bool, device=device
    )
    legal_action_mask = torch.ones(
        (batch_size, num_actions), dtype=torch.bool, device=device
    )

    class OnnxWrapper(torch.nn.Module):
        def __init__(self, inner: torch.nn.Module):
            super().__init__()
            self.inner = inner

        def forward(
            self,
            cell_features,
            global_features,
            history_features,
            history_mask,
            legal_action_mask,
        ):
            outputs = self.inner(
                cell_features=cell_features,
                global_features=global_features,
                history_features=history_features,
                history_mask=history_mask,
                legal_action_mask=legal_action_mask,
            )
            return outputs["policy_logits"], outputs["value"], outputs["wdl_logits"]

    wrapper = OnnxWrapper(model)
    wrapper.eval()
    output_path.parent.mkdir(parents=True, exist_ok=True)

    export_kwargs = dict(
        export_params=True,
        opset_version=args.opset,
        do_constant_folding=True,
        input_names=[
            "cell_features",
            "global_features",
            "history_features",
            "history_mask",
            "legal_action_mask",
        ],
        output_names=[
            "policy_logits",
            "value",
            "wdl_logits",
        ],
    )

    if args.opset >= 18:
        export_kwargs["dynamic_shapes"] = {
            "cell_features": {0: "batch"},
            "global_features": {0: "batch"},
            "history_features": {0: "batch", 1: "history_len"},
            "history_mask": {0: "batch", 1: "history_len"},
            "legal_action_mask": {0: "batch"},
        }
        export_kwargs["dynamo"] = True
    else:
        export_kwargs["dynamic_axes"] = {
            "cell_features": {0: "batch"},
            "global_features": {0: "batch"},
            "history_features": {0: "batch", 1: "history_len"},
            "history_mask": {0: "batch", 1: "history_len"},
            "legal_action_mask": {0: "batch"},
            "policy_logits": {0: "batch"},
            "value": {0: "batch"},
            "wdl_logits": {0: "batch"},
        }

    with torch.no_grad():
        torch.onnx.export(
            wrapper,
            (
                cell_features,
                global_features,
                history_features,
                history_mask,
                legal_action_mask,
            ),
            str(output_path),
            **export_kwargs,
        )

    print(f"exported ONNX model to: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
