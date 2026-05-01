import os
import sys
import argparse
import joblib
import numpy as np
import pandas as pd
from sklearn.metrics import confusion_matrix
from sklearn.preprocessing import PolynomialFeatures

MODEL_DIR = 'machine_learning/saved_model/'


def engineer_features(df, target_col='is_high_risk'):
    df_fe = df.copy()

    df_fe['operator_operand_ratio'] = df_fe['halstead_operators'] / \
        (df_fe['halstead_operands'] + 1)
    df_fe['sloc_per_function'] = df_fe['loc_sloc'] / \
        (df_fe['nom_functions'] + 1)
    df_fe['complexity_per_function'] = df_fe['wmc_cyclomatic'] / \
        (df_fe['nom_functions'] + 1)
    df_fe['effort_per_volume'] = df_fe['halstead_effort'] / \
        (df_fe['halstead_volume'] + 1)
    df_fe['avg_exit_per_fn'] = df_fe['nexits_exit_sum'] / \
        (df_fe['nom_functions'] + 1)
    df_fe['cognitive_per_cyclomatic'] = df_fe['cognitive_sum'] / \
        (df_fe['cyclomatic_cyclomatic_sum'] + 1)

    for col in ['halstead_volume', 'halstead_effort', 'loc_sloc', 'loc_ploc', 'halstead_operators', 'halstead_operands']:
        df_fe[f'{col}_log'] = np.log1p(df_fe[col])

    df_fe['repo'] = df_fe['file_path'].str.split('__').str[0]

    exclude_cols = ['file_path', 'repo', target_col]
    feature_cols = [col for col in df_fe.columns if col not in exclude_cols]

    df_fe.replace([np.inf, -np.inf], np.nan, inplace=True)
    df_fe.dropna(inplace=True)

    X = df_fe[feature_cols].copy()
    X = X.select_dtypes(include=[np.number])
    X = X.fillna(X.median())

    y = df_fe[target_col].astype(str).str.lower()

    key_features = ['halstead_difficulty', 'halstead_effort', 'halstead_volume',
                    'wmc_cyclomatic', 'cyclomatic_cyclomatic', 'loc_sloc',
                    'nom_functions', 'cognitive']
    existing_features = [f for f in key_features if f in X.columns]

    poly = PolynomialFeatures(
        degree=2, include_bias=False, interaction_only=False)
    X_key = X[existing_features].copy()
    poly_features = poly.fit_transform(X_key)
    poly_feature_names = poly.get_feature_names_out(existing_features)

    for i, name in enumerate(poly_feature_names):
        clean_name = name.replace(' ', '_')
        X[f'poly_{clean_name}'] = poly_features[:, i]

    X.replace([np.inf, -np.inf], np.nan, inplace=True)
    X = X.fillna(X.median())

    return X, y, poly


def main():
    parser = argparse.ArgumentParser(
        description='Validate model on held-out dataset')
    parser.add_argument('validation_csv', help='Path to validation CSV file')
    args = parser.parse_args()

    if not os.path.exists(args.validation_csv):
        print(f"Error: File not found: {args.validation_csv}")
        sys.exit(1)

    df_val = pd.read_csv(args.validation_csv)
    print(f"Validation data: {df_val.shape[0]} samples")

    X_val, y_val, _ = engineer_features(df_val)
    print(f"Validation feature engineered: {X_val.shape}")

    print("\n" + "="*60)
    print("VALIDATION ON HELD-OUT DATASET")
    print("="*60)

    rf_clf = joblib.load(f'{MODEL_DIR}/rf_model_smote.joblib')
    le = joblib.load(f'{MODEL_DIR}/label_encoder.joblib')

    print(f"Target classes: {le.classes_}")

    with open(f'{MODEL_DIR}/selected_features.txt', 'r') as f:
        selected_feature_names = [line.strip() for line in f.readlines()]

    X_val_final = X_val[selected_feature_names]

    print(f"Validation features: {X_val_final.shape}")

    y_pred = rf_clf.predict(X_val_final)

    y_val_true_enc = le.transform(y_val.values)
    cm = confusion_matrix(y_val_true_enc, y_pred)

    tn = cm[0, 0]
    fp = cm[0, 1]
    fn = cm[1, 0]
    tp = cm[1, 1]

    total = tp + tn + fp + fn
    accuracy = (tp + tn) / total * 100 if total > 0 else 0.0

    print(f"\n{'='*50}")
    print(f"Validation Results (RF with SMOTE)")
    print(f"{'='*50}")
    print(f"Matched files: {total}")
    print()
    print("Agreements:")
    print(f"  Both high-risk (TP): {tp}")
    print(f"  Both low-risk (TN): {tn}")
    print(f"  Disagreements:")
    print(f"    model=low, actual=high (FN): {fn}")
    print(f"    model=high, actual=low (FP): {fp}")
    print()
    print(f"Agreement (Accuracy): {accuracy:.1f}%")


if __name__ == '__main__':
    main()
