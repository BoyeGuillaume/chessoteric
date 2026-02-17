import shutil
import subprocess
from os.path import join, dirname, abspath

features = [
    "alpha_beta_soft_pruning",
]

def ask_features():
    print("Available features:")
    for i, feature in enumerate(features):
        print(f"{i + 1}. {feature}")
    selected_indices = input("Enter the numbers of the features you want to enable (comma-separated): ")
    selected_features = [features[int(i) - 1] for i in selected_indices.split(",") if i.strip().isdigit() and 1 <= int(i.strip()) <= len(features)]
    return selected_features

# 1. Build the engine and get the path to the executable
WORKSPACE_PATH = dirname(dirname(abspath(__file__)))
ENGINE_PATH = join(WORKSPACE_PATH, "target", "release", "sterm")
ENGINE_REF_PATH = f"{ENGINE_PATH}_old"

print("Select features to enable for the reference version:")
reference_features = ask_features()
build_args = ["--release", "--no-default-features"]
if reference_features:
    build_args += ["--features", ",".join(reference_features)]
subprocess.run(["cargo", "build", "--bin", "sterm", *build_args], cwd=WORKSPACE_PATH).check_returncode()
shutil.copy(ENGINE_PATH, f"{ENGINE_REF_PATH}")

# 3. Ask the user to undo the last commit/toggle the last change
print("\nSelect features to enable for the new version:")
new_features = ask_features()
build_args = ["--release", "--no-default-features"]
if new_features:
    build_args += ["--features", ",".join(new_features)]
subprocess.run(["cargo", "build", "--bin", "sterm", *build_args], cwd=WORKSPACE_PATH).check_returncode()

# 5. Finally run
OPENING_BOOK_PATH = join(WORKSPACE_PATH, "scripts", "data", "UHO_Lichess_4852_v1.epd")
args = [
    "fastchess",
    "-engine",
    f"cmd={ENGINE_REF_PATH}", f"name=Reference engine (features: {', '.join(reference_features) if reference_features else 'none'})",
    "-engine",
    f"cmd={ENGINE_PATH}", f"name=Engine (features: {', '.join(new_features) if new_features else 'none'})",
    "-each", f"tc=40/4", # 250 ms per move
    "-rounds", "500",
    "-recover",
    "-repeat",
    "-concurrency", "10",
    "-sprt", "elo0=0", "elo1=5", "alpha=0.05", "beta=0.05",
    "-openings", f"file={OPENING_BOOK_PATH}", "format=epd",
    "-log", "file=tournament_results.log", "append=true", "level=trace", "compress=false",
]
subprocess.run(args, check=True)
# print(" ".join(args))V
