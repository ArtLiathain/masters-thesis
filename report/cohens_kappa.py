def cohens_kappa(confusion_matrix):
    """
    Calculate Cohen's Kappa from a 2x2 confusion matrix.

    Args:
        confusion_matrix: 2x2 list [[TP, FP], [FN, TN]] or similar layout
                         Rows = actual, Cols = predicted

    Returns:
        kappa: Cohen's kappa coefficient
    """
    n = sum(sum(row) for row in confusion_matrix)
    po = sum(confusion_matrix[i][i] for i in range(len(confusion_matrix))) / n

    row_totals = [sum(row) for row in confusion_matrix]
    col_totals = [sum(confusion_matrix[i][j] for i in range(
        len(confusion_matrix))) for j in range(len(confusion_matrix[0]))]

    pe = sum((row_totals[i] * col_totals[i])
             for i in range(len(row_totals))) / (n * n)

    if pe == 1:
        return 1.0

    kappa = (po - pe) / (1 - pe)
    return kappa


if __name__ == "__main__":
    print("Enter 2x2 confusion matrix values:")
    print("Layout: [[TP, FP], [FN, TN]]")
    print()
    
    matrix = [
        [int(input("TP (true positive): ")), int(input("FP (false positive): "))],
        [int(input("FN (false negative): ")), int(input("TN (true negative): "))]
    ]
    
    print(f"\nConfusion matrix: {matrix}")
    print(f"Cohen's Kappa: {cohens_kappa(matrix):.4f}")
