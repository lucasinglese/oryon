"""MkDocs hooks — inject project metadata into template context."""

import re
from pathlib import Path

ROOT = Path(__file__).parent.parent


def on_config(config):
    """Inject version, feature count and target count into config.extra."""
    content = (ROOT / "pyproject.toml").read_text()
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if match:
        config.extra["version"] = match.group(1)

    features_dir = ROOT / "crates/oryon/src/features"
    operators_dir = ROOT / "crates/oryon/src/operators"
    scalers_dir = ROOT / "crates/oryon/src/scalers"
    targets_dir = ROOT / "crates/oryon/src/targets"
    feature_count = len([f for f in features_dir.rglob("*.rs") if f.name != "mod.rs"])
    operator_count = len([f for f in operators_dir.rglob("*.rs") if f.name != "mod.rs"])
    scaler_count = len([f for f in scalers_dir.rglob("*.rs") if f.name != "mod.rs"])
    config.extra["streaming_transform_count"] = feature_count + operator_count + scaler_count
    config.extra["target_count"] = len(
        [f for f in targets_dir.rglob("*.rs") if f.name != "mod.rs"]
    )
    return config
