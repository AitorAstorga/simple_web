#!/bin/bash
set -e

echo "Running Hurl API tests..."
echo "========================="

for test_file in *.hurl; do
    echo "Running $test_file..."
    if /usr/bin/hurl --test "$test_file"; then
        echo "✅ $test_file passed"
    else
        echo "❌ $test_file failed"
        exit 1
    fi
    echo ""
done

echo "All tests completed successfully!"