# Indexing Strategies Evaluation

This document summarizes an evaluation of approximate nearest neighbour indexing strategies for the memory module. The focus is on two popular approaches: **IVFADC** and **SCANN**.

## IVFADC

Inverted File with Asymmetric Distance Computation (IVFADC) is part of the FAISS library and combines an inverted file coarse quantizer with product quantization for residuals.

**Pros**
- Mature implementation in FAISS with CPU and GPU support.
- Good balance between search speed and accuracy when tuned correctly.
- Memory usage can be controlled via the number of centroids and quantization bits.

**Cons**
- Requires an explicit training phase to compute centroids and codebooks.
- Choosing the number of lists and PQ parameters impacts both recall and speed.

## SCANN

Scalable Nearest Neighbors (SCANN) from Google uses learned partitioning and quantization techniques to achieve high recall with efficient search.

**Pros**
- High quality results with competitive recall on large datasets.
- Hybrid search mode can leverage both exact and approximate search paths.
- Provides both CPU and GPU implementations.

**Cons**
- Build and integration can be complex outside of Google environments.
- Currently less flexible than FAISS for custom distance metrics.

## Comparison

Both methods provide substantial speedups over brute-force search. IVFADC is well suited when FAISS is already a dependency and offers fine-grained tuning of memory footprint. SCANN provides strong out-of-the-box recall but may involve heavier dependencies. For this project, IVFADC via FAISS remains the default, while SCANN could be explored as an optional feature if higher recall is needed.
