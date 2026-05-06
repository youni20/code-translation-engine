#!/bin/bash
 
echo "Writing Output Contents To outputs/output.txt"

cd ".."

uv run "src/main.py" > "outputs/output.txt" 2>&1

echo "Done"