# Installation

Oryon requires **Python 3.9+**. No Rust toolchain needed. Pre-built wheels
are available for Linux, macOS, and Windows.

---

## Install from PyPI

=== "pip"

    ```bash
    pip install oryon
    ```

=== "uv"

    ```bash
    uv add oryon
    ```

=== "Poetry"

    ```bash
    poetry add oryon
    ```

---

## Verify the installation

```python
import oryon
from oryon.features import Sma
from oryon import FeaturePipeline

sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
fp = FeaturePipeline(features=[sma], input_columns=["close"])
print(fp.output_names())  # ['close_sma_3']
```

---

## Development install

Requires a stable Rust toolchain (`rustup`) and `maturin`.

```bash
git clone https://github.com/lucasinglese/oryon.git
cd oryon
pip install maturin pytest
maturin develop --release
pytest
```

Or with `make`:

```bash
make test   # build + cargo test + pytest
make lint   # fmt + clippy + doc
```