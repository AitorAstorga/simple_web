#!/bin/bash

# Simple Web API Testing Script with Hurl
# This script provides easy commands to run API tests

set -e

show_help() {
    echo "Simple Web API Testing with Hurl"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start       Start the application and wait for it to be ready"
    echo "  test        Run all Hurl tests"
    echo "  test-auth   Run authentication tests only"
    echo "  test-files  Run file management tests only"
    echo "  test-git    Run git operations tests only"
    echo "  test-themes Run theme management tests only"
    echo "  stop        Stop the application"
    echo "  logs        Show application logs"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start && $0 test    # Start app and run all tests"
    echo "  $0 test-auth          # Run only auth tests"
}

start_app() {
    echo "Starting Simple Web application..."
    docker compose -f docker-compose.dev.yml up -d
    echo "Waiting for application to be ready..."

    # Wait for health check to pass
    echo "Checking application health..."
    max_attempts=30
    attempt=1

    while [ $attempt -le $max_attempts ]; do
        if docker compose -f docker-compose.dev.yml ps simple_web | grep -q "healthy"; then
            echo "✅ Application is ready!"
            return 0
        fi
        echo "Attempt $attempt/$max_attempts - waiting for application to be ready..."
        sleep 2
        attempt=$((attempt + 1))
    done

    echo "❌ Application failed to start within expected time"
    echo "Check logs with: $0 logs"
    exit 1
}

run_hurl_test() {
    local test_file="$1"
    local test_name="$2"

    echo "Running $test_name tests..."
    if docker compose --profile test -f docker-compose.dev.yml run --rm hurl_tests --test "$test_file"; then
        echo "✅ $test_name tests passed"
    else
        echo "❌ $test_name tests failed"
        return 1
    fi
}

run_all_tests() {
    echo "Running all API tests..."

    # Run each test file individually
    for test_file in auth.hurl files.hurl git.hurl themes.hurl; do
        echo "Running $test_file..."
        if docker compose --profile test -f docker-compose.dev.yml run --rm hurl_tests --test "$test_file"; then
            echo "✅ $test_file passed"
        else
            echo "❌ $test_file failed"
            return 1
        fi
    done
    echo "✅ All tests passed"
}

case "${1:-help}" in
    start)
        start_app
        ;;
    test)
        run_all_tests
        ;;
    test-auth)
        run_hurl_test "auth.hurl" "Authentication"
        ;;
    test-files)
        run_hurl_test "files.hurl" "File Management"
        ;;
    test-git)
        run_hurl_test "git.hurl" "Git Operations"
        ;;
    test-themes)
        run_hurl_test "themes.hurl" "Theme Management"
        ;;
    stop)
        echo "Stopping Simple Web application..."
        docker compose -f docker-compose.dev.yml down
        ;;
    logs)
        docker compose -f docker-compose.dev.yml logs -f simple_web
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac