import os
import json
import matplotlib.pyplot as plt
from matplotlib.ticker import LogLocator

from matplotlib.ticker import ScalarFormatter

def load_json(path):
    """Carga un archivo JSON y lo retorna como dict."""
    with open(path, "r") as f:
        return json.load(f)

class Bench:
    def __init__(self, type, crate, category, value):
        self.type = type
        self.crate = crate
        self.category = category
        self.value = value

##### MAIN #####

cmp_categories = [ "simple", "var", "var&fn", "complex" ]
cmp_types = [ "parse", "eval" ]
cmp_crates = [ "fee", "fasteval", "meval", "evalexpr" ]

root = "target/criterion"
plots_dir = os.path.join(".", "plots")
os.makedirs(plots_dir, exist_ok=True)

data = []
for cmp_dir in [d for d in os.listdir(root) if d.startswith("cmp_")]:
    parts = cmp_dir.split("_")
    _, type, crate, category = parts
    data.append(Bench(type, crate, category, load_json(os.path.join(root, cmp_dir, "new", "estimates.json"))["median"]["point_estimate"]))

if data:
    for t in cmp_types:
        subset = [d for d in data if d.type == t]

        values = []
        for cat in cmp_categories:
            row = []
            for crate in cmp_crates:
                bench = next((d for d in subset if d.category == cat and d.crate == crate), None)
                row.append(bench.value if bench else None)
            values.append(row)

        x = range(len(cmp_categories))
        bar_width = 0.2
        plt.figure(figsize=(14, 6))

        for i, crate in enumerate(cmp_crates):
            plt.bar(
                [xi + i * bar_width for xi in x],
                [row[i] for row in values],
                width=bar_width,
                label=crate,
                zorder=10
            )

        plt.xticks(
            [xi + bar_width * (len(cmp_crates) / 2) for xi in x],
            cmp_categories,
            rotation=45,
        )
        
        plt.yscale("log")
        
        plt.gca().yaxis.set_major_locator(LogLocator(base=10.0, numticks=10))
        plt.gca().yaxis.set_minor_locator(LogLocator(base=10.0, subs=range(2, 10)))
        plt.gca().yaxis.set_major_formatter(ScalarFormatter())
        plt.ylabel("Median (ns)")
        
        plt.grid(True, which="both", axis="y", linestyle="--", linewidth=0.7, color="gray", alpha=1)
        plt.title(f"{t.capitalize()} benches")
        plt.legend()
        plt.tight_layout()

        output_file = os.path.join(plots_dir, f"cmp_{t}_bench.png")
        plt.savefig(output_file, dpi=150)
