# %% [markdown]
# # Molten ML Test
# Run the cells below to check PyTorch and Plotting.

# %%
import torch
import torch.nn as nn
import matplotlib
matplotlib.use('module://matplotlib_inline.backend_inline')
import matplotlib.pyplot as plt
import numpy as np

# Check if CUDA (GPU) is available for your Master's work
device = "cuda" if torch.cuda.is_available() else "cpu"
print(f"Using device: {device}")

# %%
# 1. Setup Data (Simple Linear Regression: y = 2x + 1)
X = torch.randn(100, 1).to(device)
y = 2 * X + 1 + torch.randn(100, 1) * 0.1 # Add some noise

# 2. Simple Model
model = nn.Linear(1, 1).to(device)
criterion = nn.MSELoss()
optimizer = torch.optim.SGD(model.parameters(), lr=0.01)

# %%
# 3. Training Loop
losses = []
for epoch in range(100):
    outputs = model(X)
    loss = criterion(outputs, y)
    
    optimizer.zero_grad()
    loss.backward()
    optimizer.step()
    
    losses.append(loss.item())
    if (epoch+1) % 20 == 0:
        print(f'Epoch [{epoch+1}/100], Loss: {loss.item():.4f}')

# %%
# 4. Plot Results (The "Molten Moment")
plt.figure(figsize=(8, 4))
plt.plot(losses, label='Training Loss')
plt.title('Loss Curve')
plt.xlabel('Epoch')
plt.ylabel('MSE')
plt.legend()
plt.show()

