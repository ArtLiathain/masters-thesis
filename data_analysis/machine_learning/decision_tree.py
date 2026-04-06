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
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import GridSearchCV
from sklearn.feature_selection import SelectKBest, f_classif
import seaborn as sns
import matplotlib.pyplot as plt
from sklearn.tree import plot_tree, export_text
import pandas as pd
import numpy as np
from sklearn.model_selection import cross_val_score, cross_val_predict
from sklearn.tree import DecisionTreeClassifier
from sklearn.metrics import confusion_matrix
from sklearn.preprocessing import LabelEncoder

# %%
df = pd.read_csv('../../results/metrics.csv')
print(f"Dataset shape: {df.shape}")
print(f"\nColumns: {list(df.columns)}")
print(f"\nTarget distribution:\n{df['is_high_risk'].value_counts()}")

# %%
target_col = 'is_high_risk'
exclude_cols = ['file_path', target_col]

feature_cols = [col for col in df.columns if col not in exclude_cols]
print(f"\nNumber of features: {len(feature_cols)}")

df.replace([np.inf, -np.inf], np.nan, inplace=True)
df.dropna(inplace=True)

X = df[feature_cols].copy()
y = df[target_col].copy()

X = X.select_dtypes(include=[np.number])
print(f"Numeric features: {list(X.columns)}")

X = X.fillna(X.median())

le = LabelEncoder()
y_encoded = le.fit_transform(y.astype(str))
print(f"\nTarget classes: {le.classes_}")
print(f"Encoded: {dict(zip(le.classes_, range(len(le.classes_))))}")

# %%
df_fe = df.copy()

df_fe['operator_operand_ratio'] = df_fe['halstead_operators'] / \
    (df_fe['halstead_operands'] + 1)
df_fe['sloc_per_function'] = df_fe['loc_sloc'] / (df_fe['nom_functions'] + 1)
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

numeric_df = df_fe.select_dtypes(include=[np.number])
zero_var_cols = numeric_df.columns[numeric_df.std() == 0].tolist()
df_fe = df_fe.drop(columns=zero_var_cols)
print(f"Dropped zero-variance columns: {zero_var_cols}")

target_col = 'is_high_risk'
exclude_cols = ['file_path', target_col]

feature_cols = [col for col in df_fe.columns if col not in exclude_cols]
print(f"\nNumber of features after FE: {len(feature_cols)}")

df_fe.replace([np.inf, -np.inf], np.nan, inplace=True)
df_fe.dropna(inplace=True)

X = df_fe[feature_cols].copy()
X = X.select_dtypes(include=[np.number])
X = X.fillna(X.median())

y = df_fe[target_col].copy()
le = LabelEncoder()
y_encoded = le.fit_transform(y.astype(str))

print(f"Feature engineered dataset shape: {X.shape}")

# %%

print("\n" + "="*60)
print("PHASE 1: ENHANCED FEATURE ENGINEERING")
print("="*60)

X_enhanced = X.copy()

key_features = ['halstead_difficulty', 'halstead_effort', 'halstead_volume',
                'wmc_cyclomatic', 'cyclomatic_cyclomatic', 'loc_sloc',
                'mi_mi_original', 'mi_mi_sei', 'nom_functions', 'cognitive']

existing_features = [f for f in key_features if f in X_enhanced.columns]
print(f"Found {len(existing_features)} key features for interactions")

for i, f1 in enumerate(existing_features):
    for f2 in existing_features[i+1:]:
        if f1 != f2:
            X_enhanced[f'{f1}_x_{f2}'] = X_enhanced[f1] * X_enhanced[f2]

ratio_pairs = [
    ('halstead_effort', 'loc_sloc'),
    ('wmc_cyclomatic', 'nom_functions'),
    ('cognitive_sum', 'cyclomatic_cyclomatic_sum'),
    ('halstead_difficulty', 'halstead_volume'),
    ('nexits_exit_sum', 'nom_functions'),
    ('loc_sloc', 'loc_ploc'),
]

for f1, f2 in ratio_pairs:
    if f1 in X_enhanced.columns and f2 in X_enhanced.columns:
        X_enhanced[f'{f1}_div_{f2}'] = X_enhanced[f1] / (X_enhanced[f2] + 1)

percentile_cols = ['halstead_difficulty', 'wmc_cyclomatic', 'loc_sloc', 
                   'halstead_effort', 'cognitive_sum']
for col in percentile_cols:
    if col in X_enhanced.columns:
        X_enhanced[f'{col}_high'] = (X_enhanced[col] > X_enhanced[col].quantile(0.75)).astype(int)
        X_enhanced[f'{col}_low'] = (X_enhanced[col] < X_enhanced[col].quantile(0.25)).astype(int)

for col in percentile_cols:
    if col in X_enhanced.columns:
        q75, q25 = X_enhanced[col].quantile(0.75), X_enhanced[col].quantile(0.25)
        iqr = q75 - q25
        upper = q75 + 1.5 * iqr
        X_enhanced[f'{col}_outlier'] = (X_enhanced[col] > upper).astype(int)

X_enhanced.replace([np.inf, -np.inf], np.nan, inplace=True)
X_enhanced = X_enhanced.fillna(X_enhanced.median())

print(f"Enhanced feature count: {X_enhanced.shape[1]}")

# %%

print("\n" + "="*60)
print("PHASE 2: FEATURE SELECTION")
print("="*60)

corr_matrix = X_enhanced.corr().abs()
upper = corr_matrix.where(np.triu(np.ones(corr_matrix.shape), k=1).astype(bool))
to_drop = [column for column in upper.columns if any(upper[column] > 0.90)]
print(f"Dropping {len(to_drop)} highly correlated features (>0.90)")
X_selected = X_enhanced.drop(columns=to_drop)

selector = SelectKBest(f_classif, k=min(30, X_selected.shape[1]))
X_final = pd.DataFrame(
    selector.fit_transform(X_selected, y_encoded),
    columns=[X_selected.columns[i] for i in selector.get_support(indices=True)],
    index=X_selected.index
)

print(f"Final feature count after selection: {X_final.shape[1]}")
print(f"Selected features: {list(X_final.columns)}")

cv = 5

# %%

print("\n" + "="*60)
print("PHASE 3: RANDOM FOREST WITH F1 OPTIMIZATION")
print("="*60)

rf_param_grid = {
    'n_estimators': [300, 500, 700],
    'max_depth': [5, 10, 15, 20, None],
    'min_samples_split': [2, 5, 10],
    'min_samples_leaf': [1, 2, 4],
    'max_features': ['sqrt', 'log2', None],
    'class_weight': ['balanced', {0: 1, 1: 1.5}, {0: 1, 1: 2}]
}

rf_clf = RandomForestClassifier(random_state=42, n_jobs=-1, oob_score=True)

rf_grid_search = GridSearchCV(
    rf_clf, rf_param_grid, cv=cv, scoring='f1', n_jobs=-1, verbose=1
)
rf_grid_search.fit(X_final, y_encoded)

print("Best Random Forest params:", rf_grid_search.best_params_)
print("Best CV F1 score:", rf_grid_search.best_score_)

rf_clf = rf_grid_search.best_estimator_

rf_accuracy_scores = cross_val_score(
    rf_clf, X_final, y_encoded, cv=cv, scoring='accuracy')
rf_precision_scores = cross_val_score(
    rf_clf, X_final, y_encoded, cv=cv, scoring='precision')
rf_recall_scores = cross_val_score(
    rf_clf, X_final, y_encoded, cv=cv, scoring='recall')
rf_f1_scores = cross_val_score(rf_clf, X_final, y_encoded, cv=cv, scoring='f1')

print("Random Forest 5-Fold Cross-Validation Results (F1 optimized)")
print("=" * 50)
print(f"Accuracy:  {rf_accuracy_scores.mean():.4f} (+/- {rf_accuracy_scores.std():.4f})")
print(f"Precision: {rf_precision_scores.mean():.4f} (+/- {rf_precision_scores.std():.4f})")
print(f"Recall:    {rf_recall_scores.mean():.4f} (+/- {rf_recall_scores.std():.4f})")
print(f"F1 Score:  {rf_f1_scores.mean():.4f} (+/- {rf_f1_scores.std():.4f})")

rf_clf.fit(X_final, y_encoded)

rf_y_pred = cross_val_predict(rf_clf, X_final, y_encoded, cv=cv)
rf_cm = confusion_matrix(y_encoded, rf_y_pred)
print("\nRF Confusion Matrix:")
print(rf_cm)

rf_feature_importance = pd.DataFrame({
    'feature': X_final.columns,
    'importance': rf_clf.feature_importances_
}).sort_values('importance', ascending=False)
print("\nRF Feature Importances (top 15):")
print(rf_feature_importance.head(15))

print(f"\nOOB Score: {rf_clf.oob_score_:.4f}")

# %%

print("\n" + "="*60)
print("DECISION TREE WITH F1 OPTIMIZATION")
print("="*60)

dt_param_grid = {
    'max_depth': [3, 5, 7, 10, 15, None],
    'min_samples_split': [2, 5, 10, 20],
    'min_samples_leaf': [1, 2, 5, 10],
    'criterion': ['gini', 'entropy'],
    'class_weight': ['balanced', {0: 1, 1: 1.5}, {0: 1, 1: 2}]
}

dt_clf = DecisionTreeClassifier(random_state=42)

dt_grid_search = GridSearchCV(
    dt_clf, dt_param_grid, cv=cv, scoring='f1', n_jobs=-1, verbose=1
)
dt_grid_search.fit(X_final, y_encoded)

print("Best Decision Tree params:", dt_grid_search.best_params_)
print("Best CV F1:", dt_grid_search.best_score_)

dt_clf = dt_grid_search.best_estimator_

dt_accuracy_scores = cross_val_score(dt_clf, X_final, y_encoded, cv=cv, scoring='accuracy')
dt_precision_scores = cross_val_score(dt_clf, X_final, y_encoded, cv=cv, scoring='precision')
dt_recall_scores = cross_val_score(dt_clf, X_final, y_encoded, cv=cv, scoring='recall')
dt_f1_scores = cross_val_score(dt_clf, X_final, y_encoded, cv=cv, scoring='f1')

print("Decision Tree 5-Fold Cross-Validation Results")
print("=" * 50)
print(f"Accuracy:  {dt_accuracy_scores.mean():.4f} (+/- {dt_accuracy_scores.std():.4f})")
print(f"Precision: {dt_precision_scores.mean():.4f} (+/- {dt_precision_scores.std():.4f})")
print(f"Recall:    {dt_recall_scores.mean():.4f} (+/- {dt_recall_scores.std():.4f})")
print(f"F1 Score:  {dt_f1_scores.mean():.4f} (+/- {dt_f1_scores.std():.4f})")

# %%

dt_clf.fit(X_final, y_encoded)

dt_y_pred = cross_val_predict(dt_clf, X_final, y_encoded, cv=cv)
dt_cm = confusion_matrix(y_encoded, dt_y_pred)
print("DT Confusion Matrix:")
print(dt_cm)

dt_feature_importance = pd.DataFrame({
    'feature': X_final.columns,
    'importance': dt_clf.feature_importances_
}).sort_values('importance', ascending=False)
print("\nDT Feature Importances (top 15):")
print(dt_feature_importance.head(15))

# %%

fig, ax = plt.subplots(figsize=(24, 12))
plot_tree(dt_clf, feature_names=list(X_final.columns),
          class_names=list(le.classes_), filled=True, ax=ax, max_depth=4)
plt.title("Decision Tree Classifier (max_depth=4 for readability)")
plt.tight_layout()
plt.savefig("decision_tree_optimized.png", dpi=150, bbox_inches='tight')
plt.show()
print("Tree visualization saved to decision_tree_optimized.png")

# %%

print("\n" + "="*60)
print("EXTRACTING DECISION TREE RULES (INTERPRETABLE)")
print("="*60)

tree_rules = export_text(dt_clf, feature_names=list(X_final.columns), max_depth=4)
print("Decision Tree Rules (first 4 levels):")
print(tree_rules)

# %%

print("\n" + "="*60)
print("MODEL COMPARISON")
print("="*60)

plt.figure(figsize=(12, 6))
models = ['Random Forest', 'Decision Tree']
f1_scores_plot = [rf_f1_scores.mean(), dt_f1_scores.mean()]
colors = ['#3498db', '#e74c3c']

plt.bar(models, f1_scores_plot, color=colors)
plt.ylabel('F1 Score')
plt.title('Model Comparison (F1 Score)')
plt.ylim(0, 1)
for i, v in enumerate(f1_scores_plot):
    plt.text(i, v + 0.02, f'{v:.4f}', ha='center', fontweight='bold')

plt.tight_layout()
plt.savefig('model_comparison.png', dpi=150, bbox_inches='tight')
plt.show()

# %%

print("\n" + "="*60)
print("SUMMARY")
print("="*60)
print(f"Original features: {X.shape[1]}")
print(f"Enhanced features: {X_enhanced.shape[1]}")
print(f"Selected features: {X_final.shape[1]}")
print(f"\nRandom Forest F1: {rf_f1_scores.mean():.4f} (+/- {rf_f1_scores.std():.4f})")
print(f"Decision Tree F1: {dt_f1_scores.mean():.4f} (+/- {dt_f1_scores.std():.4f})")

print("\n" + "="*60)
print("OPTIMIZATION COMPLETE")
print("="*60)

print("\nFinal Feature Set:")
print(X_final.columns.tolist())
