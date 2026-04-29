# ---
# jupyter:
#   jupytext:
#     cell_metadata_filter: -all
#     formats: ipynb,py:percent
#     text_representation:
#       extension: .py
#       format_name: percent
#       format_version: '1.3'
#       jupytext_version: 1.19.1
#   kernelspec:
#     display_name: Python (ML Project)
#     language: python
#     name: ml-project
# ---

# %%
import os
import joblib
import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import GridSearchCV, StratifiedKFold, cross_val_score, cross_val_predict
from sklearn.feature_selection import SelectKBest, f_classif, VarianceThreshold
from sklearn.tree import DecisionTreeClassifier, plot_tree, export_text
from sklearn.metrics import confusion_matrix, accuracy_score, precision_score, recall_score, f1_score
from sklearn.preprocessing import LabelEncoder, PolynomialFeatures
from imblearn.over_sampling import SMOTE
from te2rules.explainer import ModelExplainer
from sklearn.model_selection import train_test_split
from imblearn.pipeline import Pipeline as ImbPipeline

# %%
TRAIN_MODEL = True

TRAIN_CSV = '../../results/metrics.csv'
VAL_CSV = '../../results/codescene_metrics.csv'

MODEL_OUTPUT_DIR = 'saved_model/'
MODEL_DIR = 'saved_model/'
OUTPUT_PNG = '../../results/acts_validation_predictions.png'

# %%


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


# %%
def select_features(X, y, var_thresh=None, selector=None):
    corr_matrix = X.corr().abs()
    upper = corr_matrix.where(
        np.triu(np.ones(corr_matrix.shape), k=1).astype(bool))
    to_drop = [column for column in upper.columns if any(upper[column] > 0.90)]
    X_selected = X.drop(columns=to_drop)

    if var_thresh is None:
        var_thresh = VarianceThreshold(threshold=0.01)
        X_transformed = var_thresh.fit_transform(X_selected)
    else:
        X_transformed = var_thresh.transform(X_selected)

    X_selected = pd.DataFrame(
        X_transformed,
        columns=X_selected.columns[var_thresh.get_support()],
        index=X_selected.index
    )

    X_selected.replace([np.inf, -np.inf], np.nan, inplace=True)
    X_selected = X_selected.fillna(X_selected.median())
    to_drop_nan = X_selected.columns[X_selected.isna().all()]
    if len(to_drop_nan) > 0:
        X_selected = X_selected.drop(columns=to_drop_nan)

    best_k = min(30, X_selected.shape[1])
    if selector is None:
        selector = SelectKBest(f_classif, k=best_k)
        X_final = pd.DataFrame(
            selector.fit_transform(X_selected, y),
            columns=[X_selected.columns[i]
                     for i in selector.get_support(indices=True)],
            index=X_selected.index
        )
    else:
        X_final = pd.DataFrame(
            selector.transform(X_selected),
            columns=[X_selected.columns[i]
                     for i in selector.get_support(indices=True)],
            index=X_selected.index
        )

    return X_final, var_thresh, selector


# %%
if TRAIN_MODEL:
    le = LabelEncoder()

    df_train = pd.read_csv(TRAIN_CSV)
    print(f"Training data: {df_train.shape[0]} samples")

    # %%
    print("\n" + "="*60)
    print("REPO-LEVEL METRICS")
    print("="*60)

    df_train['repo'] = df_train['file_path'].str.split('__').str[0]

    repo_stats = df_train.groupby('repo').agg({
        'file_path': 'count',
        'is_high_risk': lambda x: x.sum(),
        'loc_sloc': 'sum',
        'halstead_volume': 'sum',
        'halstead_effort': 'sum',
        'wmc_cyclomatic': 'sum',
        'cyclomatic_cyclomatic': 'sum',
        'nom_functions': 'sum',
        'cognitive': 'sum'
    }).rename(columns={
        'file_path': 'file_count',
        'is_high_risk': 'high_risk_files',
        'loc_sloc': 'total_sloc',
        'halstead_volume': 'total_volume',
        'halstead_effort': 'total_effort',
        'wmc_cyclomatic': 'total_cyclomatic',
        'cyclomatic_cyclomatic': 'total_path_complexity',
        'nom_functions': 'total_functions',
        'cognitive': 'total_cognitive'
    })

    repo_stats['low_risk_files'] = repo_stats['file_count'] - repo_stats['high_risk_files']
    repo_stats['risk_ratio'] = repo_stats['high_risk_files'] / repo_stats['file_count']

    print(f"Total repos: {len(repo_stats)}")
    print(f"Total files: {repo_stats['file_count'].sum()}")
    print(f"High-risk files: {repo_stats['high_risk_files'].sum()} ({repo_stats['high_risk_files'].sum() / repo_stats['file_count'].sum():.2%})")
    print(f"Low-risk files: {repo_stats['low_risk_files'].sum()} ({repo_stats['low_risk_files'].sum() / repo_stats['file_count'].sum():.2%})")
    print(f"Average files per repo: {repo_stats['file_count'].mean():.1f}")
    print(f"Average risk ratio per repo: {repo_stats['risk_ratio'].mean():.2%}")

    print("\nTop 10 repos by high-risk file count:")
    top_risky = repo_stats.sort_values('high_risk_files', ascending=False)[['file_count', 'high_risk_files', 'low_risk_files', 'risk_ratio']].head(10)
    print(top_risky.to_string())

    print("\nTop 10 repos by risk ratio (min 5 files):")
    top_ratio = repo_stats[repo_stats['file_count'] >= 5].sort_values('risk_ratio', ascending=False)[['file_count', 'high_risk_files', 'low_risk_files', 'risk_ratio']].head(10)
    print(top_ratio.to_string())

    X_train, y_train, poly = engineer_features(df_train)

    print("\n" + "="*60)
    print("CLASS DISTRIBUTION (TRAINING DATA)")
    print("="*60)
    value_counts = y_train.value_counts()
    print(value_counts)
    print(f"\nClass imbalance ratio: {
          value_counts.max() / value_counts.min():.2f}:1")

    plt.figure(figsize=(8, 5))
    value_counts.plot(kind='bar', color=['#3498db', '#e74c3c'])
    plt.title('Class Distribution (Training Data)')
    plt.xlabel('Risk Level')
    plt.ylabel('Count')
    plt.xticks(rotation=0)
    for i, v in enumerate(value_counts):
        plt.text(i, v + 10, str(v), ha='center')
    plt.tight_layout()
    plt.savefig('class_distribution.png', dpi=150)
    plt.show()
    print("Class distribution saved to class_distribution.png")
    y_train_encoded = le.fit_transform(y_train.astype(str))
    print(f"Target classes: {le.classes_}")
    print(f"Feature engineered shape: {X_train.shape}")


# %%
if TRAIN_MODEL:
    X_train_final, var_thresh, selector = select_features(
        X_train, y_train_encoded)
    print(f"Final feature count: {X_train_final.shape[1]}")
    print(f"Selected features: {list(X_train_final.columns)}")
    print(f"Training FE columns: {sorted(X_train_final.columns.tolist())}")

    smote = SMOTE(sampling_strategy='minority', k_neighbors=5, random_state=42)
    X_train_balanced, y_train_balanced = smote.fit_resample(
        X_train_final, y_train_encoded)
    print(f"\nAfter SMOTE: {
          X_train_balanced.shape[0]} samples (was {X_train_final.shape[0]})")
    print(f"Class distribution: {pd.Series(
        y_train_balanced).value_counts().to_dict()}")


# %%
if TRAIN_MODEL:
    cv = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
    pipeline = ImbPipeline([
        ('smote', SMOTE(random_state=42)),
        ('rf', RandomForestClassifier(random_state=42))
    ])

    print("\n" + "="*60)
    print("RANDOM FOREST WITH 5-FOLD CROSS-VALIDATION")
    print("="*60)

    rf_param_grid = {
        'rf__n_estimators': [100, 300, 500],
        'rf__max_depth': [7, 10, 15, 20 , 30],
        'rf__min_samples_split': [5, 10],
        'rf__min_samples_leaf': [4, 8],
        'rf__max_features': ['sqrt', 'log2'],
        'rf__class_weight': ['balanced', {0: 1, 1: 3}, {0: 1, 1: 10}]
    }

    rf_grid_search = GridSearchCV(
        param_grid=rf_param_grid, cv=cv, estimator=pipeline, scoring='f1', n_jobs=-1
    )
    rf_grid_search.fit(X_train_final, y_train_encoded)

    print("Best Random Forest params:", rf_grid_search.best_params_)
    print("Best CV F1 score:", rf_grid_search.best_score_)

    rf_clf = rf_grid_search.best_estimator_

    cv_accuracy = cross_val_score(
        rf_clf, X_train_final, y_train_encoded, cv=cv, scoring='accuracy')
    cv_precision = cross_val_score(
        rf_clf, X_train_final, y_train_encoded, cv=cv, scoring='precision')
    cv_recall = cross_val_score(
        rf_clf, X_train_final, y_train_encoded, cv=cv, scoring='recall')
    cv_f1 = cross_val_score(
        rf_clf, X_train_final, y_train_encoded, cv=cv, scoring='f1')

    print("\nRandom Forest 5-Fold Cross-Validation Results:")
    print(f"Accuracy:  {cv_accuracy.mean():.4f} +/- {cv_accuracy.std():.4f}")
    print(f"Precision: {cv_precision.mean():.4f} +/- {cv_precision.std():.4f}")
    print(f"Recall:    {cv_recall.mean():.4f} +/- {cv_recall.std():.4f}")
    print(f"F1 Score:  {cv_f1.mean():.4f} +/- {cv_f1.std():.4f}")

    cv_preds = cross_val_predict(rf_clf, X_train_final, y_train_encoded, cv=cv)

    print("\nCross-Validation Confusion Matrix:")
    cv_cm = confusion_matrix(y_train_encoded, cv_preds)
    print(cv_cm)

    plt.figure(figsize=(8, 6))
    sns.heatmap(cv_cm, annot=True, fmt='d', cmap='Blues',
                xticklabels=le.classes_, yticklabels=le.classes_)
    plt.xlabel('Predicted')
    plt.ylabel('Actual')
    plt.title('RF Cross-Validation Confusion Matrix')
    plt.tight_layout()
    plt.savefig('rf_cv_confusion_matrix.png', dpi=150)
    plt.show()
    print("RF CV confusion matrix saved to rf_cv_confusion_matrix.png")

    rf_clf.fit(X_train_final, y_train_encoded)
    rf_best_params = rf_grid_search.best_params_
    rf_for_oob = RandomForestClassifier(
        oob_score=True,
        bootstrap=True,
        random_state=42,
        **{k.replace('rf__', ''): v for k, v in rf_best_params.items() if k.startswith('rf__')}
    )
    rf_for_oob.fit(X_train_final, y_train_encoded)
    oob_val = rf_for_oob.oob_score_
    print(f"\nOOB Score: {oob_val:.4f}")

    rf_feature_importance = pd.DataFrame({
        'feature': X_train_final.columns,
        'importance': rf_for_oob.feature_importances_
    }).sort_values('importance', ascending=False)
    print("\nRF Feature Importances (top 15):")
    print(rf_feature_importance.head(15))

    plt.figure(figsize=(10, 8))
    rf_top15 = rf_feature_importance.head(
        15).sort_values('importance', ascending=True)
    colors = sns.color_palette("viridis", 15)
    sns.barplot(x='importance', y='feature', data=rf_top15, palette=colors)
    plt.xlabel('Importance')
    plt.ylabel('Feature')
    plt.title('Random Forest Feature Importance (Top 15)')
    plt.tight_layout()
    plt.savefig('rf_feature_importance.png', dpi=150)
    plt.show()
    print("RF feature importance saved to rf_feature_importance.png")

    print(f"Target classes: {le.classes_}")

    os.makedirs(MODEL_OUTPUT_DIR, exist_ok=True)
    joblib.dump(rf_clf, f'{MODEL_OUTPUT_DIR}/rf_model.joblib')
    joblib.dump(le, f'{MODEL_OUTPUT_DIR}/label_encoder.joblib')
    joblib.dump(var_thresh, f'{MODEL_OUTPUT_DIR}/var_thresh.joblib')
    joblib.dump(selector, f'{MODEL_OUTPUT_DIR}/selector.joblib')
    with open(f'{MODEL_OUTPUT_DIR}/selected_features.txt', 'w') as f:
        f.write('\n'.join(X_train_final.columns))

    print(f"\nModel saved to {MODEL_OUTPUT_DIR}")

# %%

print("\n" + "="*60)
print("OPTIMIZED RANDOM FOREST RULE EXTRACTION (te2rules)")
print("="*60)
all_features = list(X_train_final.columns)

_, X_sub, _, y_sub = train_test_split(
    X_train_final,
    y_train_encoded,
    test_size=2000,  # Start with 1.5k; if fast, try 3k
    stratify=y_train_encoded,
    random_state=42
)

explainer = ModelExplainer(
    model=rf_clf.named_steps['rf'],
    feature_names=all_features
)

print(f"Extracting rules using {
      len(all_features)} features and {len(X_sub)} samples...")
rf_rules = explainer.explain(X_sub.values, y_sub)

print("\nExtracted Rules from Random Forest:")
print(rf_rules)


# %%
def evaluate_rules(rules_list, X_df, y_true):
    results = []
    
    for rule in rules_list:
        # Clean the rule for pandas query syntax
        # te2rules uses '&' and '>', which pandas.query handles well
        query_string = rule.replace('&', 'and')
        
        try:
            # Find samples that satisfy the rule
            matches = X_df.query(query_string)
            support = len(matches)
            
            if support > 0:
                # Calculate how many matches were actually the positive class (1)
                matching_labels = y_true[matches.index]
                precision = (matching_labels == 1).mean()
                
                results.append({
                    'rule': rule,
                    'support': support,
                    'precision': precision,
                    'coverage_pct': (support / len(X_df)) * 100
                })
        except Exception as e:
            continue # Skip rules with parsing issues
            
    return pd.DataFrame(results).sort_values(by='support', ascending=False)

# Run the evaluation
rule_stats = evaluate_rules(rf_rules, X_sub, y_sub)

print("Top 5 Most Important Rules (by Support):")
pd.set_option('display.max_colwidth', None)
print(rule_stats.head(5))
print(len(rf_rules))

# %%
print("\n" + "="*60)
print("DECISION TREE WITH 5-FOLD CROSS-VALIDATION")
print("="*60)

dt_pipeline = ImbPipeline([
    ('smote', SMOTE(random_state=42)),
    ('dt', DecisionTreeClassifier(random_state=42))
])

dt_param_grid = {
    'dt__max_depth': [3, 5, 7, 10, 15],
    'dt__min_samples_split': [5, 10, 20, 50],
    'dt__min_samples_leaf': [2, 5, 10, 20],
    'dt__criterion': ['gini', 'entropy'],
    'dt__class_weight': ['balanced', {0: 1, 1: 2}, {0: 1, 1: 3}, {0: 1, 1: 5}]
}

dt_grid_search = GridSearchCV(
    dt_pipeline, dt_param_grid, cv=cv, scoring='f1', n_jobs=-1
)
dt_grid_search.fit(X_train_final, y_train_encoded)

print("Best Decision Tree params:", dt_grid_search.best_params_)
print("Best CV F1:", dt_grid_search.best_score_)

dt_clf = dt_grid_search.best_estimator_

dt_cv_accuracy = cross_val_score(
    dt_clf, X_train_final, y_train_encoded, cv=cv, scoring='accuracy')
dt_cv_precision = cross_val_score(
    dt_clf, X_train_final, y_train_encoded, cv=cv, scoring='precision')
dt_cv_recall = cross_val_score(
    dt_clf, X_train_final, y_train_encoded, cv=cv, scoring='recall')
dt_cv_f1 = cross_val_score(dt_clf, X_train_final,
                           y_train_encoded, cv=cv, scoring='f1')

print("\nDecision Tree 5-Fold Cross-Validation Results:")
print(f"Accuracy:  {dt_cv_accuracy.mean():.4f} +/- {dt_cv_accuracy.std():.4f}")
print(f"Precision: {dt_cv_precision.mean():.4f} +/- {dt_cv_precision.std():.4f}")
print(f"Recall:    {dt_cv_recall.mean():.4f} +/- {dt_cv_recall.std():.4f}")
print(f"F1 Score:  {dt_cv_f1.mean():.4f} +/- {dt_cv_f1.std():.4f}")

dt_cv_preds = cross_val_predict(dt_clf, X_train_final, y_train_encoded, cv=cv)
dt_cv_cm = confusion_matrix(y_train_encoded, dt_cv_preds)
print("\nDT CV Confusion Matrix:")
print(dt_cv_cm)

dt_feature_importance = pd.DataFrame({
    'feature': X_train_final.columns,
    'importance': dt_clf.named_steps['dt'].feature_importances_
}).sort_values('importance', ascending=False)
print("\nDT Feature Importances (top 15):")
print(dt_feature_importance.head(15))

plt.figure(figsize=(8, 6))
sns.heatmap(dt_cv_cm, annot=True, fmt='d', cmap='Blues',
            xticklabels=le.classes_, yticklabels=le.classes_)
plt.xlabel('Predicted')
plt.ylabel('Actual')
plt.title('DT Cross-Validation Confusion Matrix')
plt.tight_layout()
plt.savefig('dt_cv_confusion_matrix.png', dpi=150)
plt.show()

plt.figure(figsize=(10, 8))
dt_top15 = dt_feature_importance.head(
    15).sort_values('importance', ascending=True)
colors = sns.color_palette("coolwarm", 15)
sns.barplot(x='importance', y='feature', data=dt_top15, palette=colors)
plt.xlabel('Importance')
plt.ylabel('Feature')
plt.title('Decision Tree Feature Importance (Top 15)')
plt.tight_layout()
plt.savefig('dt_feature_importance.png', dpi=150)
plt.show()
print("DT feature importance saved to dt_feature_importance.png")

# %%
fig, ax = plt.subplots(figsize=(24, 12))
plot_tree(dt_clf.named_steps['dt'], feature_names=list(X_train_final.columns),
          class_names=list(le.classes_), filled=True, ax=ax, max_depth=2)
plt.title("Decision Tree Classifier (max_depth=2 for readability)")
plt.tight_layout()
plt.savefig("decision_tree_optimized.png", dpi=150, bbox_inches='tight')
plt.show()
print("Tree visualization saved to decision_tree_optimized.png")

# %%
print("\n" + "="*60)
print("EXTRACTING DECISION TREE RULES (INTERPRETABLE)")
print("="*60)

tree_rules = export_text(
    dt_clf.named_steps['dt'], feature_names=list(X_train_final.columns), max_depth=4)
print("Decision Tree Rules (first 4 levels):")
print(tree_rules)

# %%
print("\n" + "="*60)
print("MODEL COMPARISON (CV-BASED)")
print("="*60)

plt.figure(figsize=(12, 6))
models = ['Random Forest', 'Decision Tree']
cv_f1_scores = [cv_f1.mean(), dt_cv_f1.mean()]
colors = ['#3498db', '#e74c3c']

plt.bar(models, cv_f1_scores, color=colors)
plt.ylabel('F1 Score')
plt.title('Model Comparison (CV F1 Score)')
plt.ylim(0, 1)
for i, v in enumerate(cv_f1_scores):
    plt.text(i, v + 0.02, f'{v:.4f}', ha='center', fontweight='bold')

plt.tight_layout()
plt.savefig('model_comparison.png', dpi=150, bbox_inches='tight')
plt.show()

# %%
print("\n" + "="*60)
print("SUMMARY")
print("="*60)
print(f"Original features: {X_train.shape[1]}")
print(f"Enhanced features: {X_train.shape[1]}")
print(f"Selected features: {X_train_final.shape[1]}")
print(f"\nRandom Forest CV F1: {cv_f1.mean():.4f} +/- {cv_f1.std():.4f}")
print(f"Decision Tree CV F1: {dt_cv_f1.mean():.4f} +/- {dt_cv_f1.std():.4f}")

print("\n" + "="*60)
print("OPTIMIZATION COMPLETE")
print("="*60)

print("\nFinal Feature Set:")
print(X_train_final.columns.tolist())


# %%
df_val = pd.read_csv(VAL_CSV)
print(f"Validation data: {df_val.shape[0]} samples")

X_val, y_val, _ = engineer_features(df_val)
print(f"Validation feature engineered: {X_val.shape}")


# %%
print("\n" + "="*60)
print("VALIDATION ON HELD-OUT DATASET")
print("="*60)

rf_clf = joblib.load(f'{MODEL_DIR}/rf_model.joblib')
le = joblib.load(f'{MODEL_DIR}/label_encoder.joblib')

print(f"Target classes: {le.classes_}")

with open(f'{MODEL_DIR}/selected_features.txt', 'r') as f:
    selected_feature_names = [line.strip() for line in f.readlines()]

# Select the same columns used in training - no sklearn selector needed
X_val_final = X_val[selected_feature_names]

print(f"Validation features: {X_val_final.shape}")

y_pred = rf_clf.predict(X_val_final)
y_prob = rf_clf.predict_proba(X_val_final)[:, 1]

y_val_true_enc = le.transform(y_val.values)
acc = accuracy_score(y_val_true_enc, y_pred)
prec = precision_score(y_val_true_enc, y_pred, zero_division=0)
rec = recall_score(y_val_true_enc, y_pred, zero_division=0)
f1 = f1_score(y_val_true_enc, y_pred, zero_division=0)
cm = confusion_matrix(y_val_true_enc, y_pred)

print(f"\nValidation Metrics:")
print(f"Accuracy:  {acc:.4f}")
print(f"Precision: {prec:.4f}")
print(f"Recall:    {rec:.4f}")
print(f"F1 Score:  {f1:.4f}")
print(f"\nConfusion Matrix:")
print(cm)

plt.figure(figsize=(8, 6))
sns.heatmap(cm, annot=True, fmt='d', cmap='Blues',
            xticklabels=le.classes_, yticbklabels=le.classes_)
plt.xlabel('Predicted')
plt.ylabel('Actual')
plt.title('Validation Confusion Matrix')
plt.tight_layout()b
plt.savefig(OUTPUT_PNG, dpi=150)
plt.show()
print(f"Confusion matrix saved to {OUTPUT_PNG}")

# %%
pd.set_option('display.max_colwidth', None)
print(rule_stats.head(5))
print(len(rf_rules))

# %%
