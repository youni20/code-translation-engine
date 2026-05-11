from pathlib import Path
from dataset import load_project, load_all_projects

if __name__ == "__main__":
    units = load_project(Path("../inputs/projects/project_a/SQLiteCpp"))
    print(f"Loaded {len(units)} files")
    for u in units[:3]:
        print(f"  {u.unit_id}: {u.loc} LOC, {len(u.source_code)} chars")