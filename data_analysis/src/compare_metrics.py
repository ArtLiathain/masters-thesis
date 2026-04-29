#!/usr/bin/env python3
"""
Compare high-risk classifications between hub_score metrics and CodeScene metrics.
"""

import argparse
import csv
import sys


def normalize_path(path):
    """Normalize path for matching - strip prefix and convert _ to /"""
    if '__' in path:
        path = path.split('__', 1)[1]
    return path.replace('_', '/')


def load_metrics(filepath):
    """Load metrics CSV and return dict mapping normalized path to is_high_risk label."""
    data = {}
    with open(filepath, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            file_path = row['file_path']
            is_high_risk = row.get('is_high_risk', 'false')
            normalized = normalize_path(file_path)
            data[normalized] = is_high_risk
    return data


def compare_metrics(hub_score_data, codescene_data):
    """Compare high-risk labels between two datasets."""
    hub_score_files = set(hub_score_data.keys())
    cs_files = set(codescene_data.keys())
    matched = hub_score_files & cs_files

    if not matched:
        return {
            'matched': 0,
            'tp': 0,
            'tn': 0,
            'fp': 0,
            'fn': 0,
            'accuracy': 0.0
        }

    tp = tn = fp = fn = 0

    for path in matched:
        hub_score_label = hub_score_data[path]
        cs_label = codescene_data[path]

        if hub_score_label == 'true' and cs_label == 'true':
            tp += 1
        elif hub_score_label == 'false' and cs_label == 'false':
            tn += 1
        elif hub_score_label == 'true' and cs_label == 'false':
            fp += 1
        elif hub_score_label == 'false' and cs_label == 'true':
            fn += 1

    total = tp + tn + fp + fn
    accuracy = (tp + tn) / total * 100 if total > 0 else 0.0

    return {
        'matched': len(matched),
        'tp': tp,
        'tn': tn,
        'fp': fp,
        'fn': fn,
        'accuracy': accuracy
    }


def main():
    parser = argparse.ArgumentParser(
        description='Compare high-risk classifications between Hub Score and CodeScene metrics'
    )
    parser.add_argument('--hub-score', required=True,
                        help='Path to Hub score metrics CSV')
    parser.add_argument('--codescene', required=True,
                        help='Path to CodeScene metrics CSV')

    args = parser.parse_args()

    try:
        hub_score_data = load_metrics(args.hub_score)
        codescene_data = load_metrics(args.codescene)
    except FileNotFoundError as e:
        print(f"Error: File not found - {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error loading files: {e}", file=sys.stderr)
        sys.exit(1)

    result = compare_metrics(hub_score_data, codescene_data)

    print("=" * 50)
    print("High-Risk Classification Comparison")
    print("=" * 50)
    print(f"Matched files: {result['matched']}")
    print()
    print("Agreements:")
    print(f"  Both high-risk (TP): {result['tp']}")
    print(f"  Both low-risk (TN): {result['tn']}")
    print(f"  Disagreements:")
    print(f"    hub_score=low, CodeScene=high (FN): {result['fn']}")
    print(f"    hub_score=high, CodeScene=low (FP): {result['fp']}")
    print()
    print(f"Agreement (Accuracy): {result['accuracy']:.1f}%")


if __name__ == '__main__':
    main()
