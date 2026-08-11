import re
import os
import sys

def extract_brief(plan_path, task_num, out_path):
    n = int(task_num)
    with open(plan_path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    
    intask = False
    infence = False
    task_lines = []
    
    print(f"Read {len(lines)} lines from plan.")
    for line in lines:
        if line.startswith("```"):
            infence = not infence
        if not infence and line.strip().startswith("### Task"):
            print(f"Found Task line: {line.strip()}")
            m = re.match(r"^### Task\s+(\d+):", line.strip())
            if m:
                print(f"Matched task number: {m.group(1)}")
                intask = (int(m.group(1)) == n)
            else:
                print("Failed to match regex")
                intask = False
        if intask:
            task_lines.append(line)
            
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.writelines(task_lines)
    print(f"Wrote {len(task_lines)} lines to {out_path}")

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python extract_brief.py <plan_path> <task_num> <out_path>")
        sys.exit(1)
    extract_brief(sys.argv[1], sys.argv[2], sys.argv[3])
