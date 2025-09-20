# API Testing with Hurl

This directory contains [Hurl](https://hurl.dev) test files for testing the Simple Web API endpoints.

## Test Files

- `auth.hurl` - Authentication endpoint tests
- `files.hurl` - File management API tests
- `git.hurl` - Git operations API tests
- `themes.hurl` - Theme management API tests

## Quick Start

### Using the Test Script (Recommended)

The easiest way to run tests is using the provided test script:

```bash
# Start the application and run all tests
./test.sh start && ./test.sh test

# Or run specific test suites
./test.sh test-auth    # Authentication tests only
./test.sh test-files   # File management tests only
./test.sh test-git     # Git operations tests only
./test.sh test-themes  # Theme management tests only

# Stop the application when done
./test.sh stop
```

### Manual Docker Compose Commands

If you prefer manual control:

```bash
# Start the application
docker compose -f docker-compose.dev.yml up -d

# Wait for the application to be healthy
docker compose -f docker-compose.dev.yml ps

# Run all tests
docker compose --profile test -f docker-compose.dev.yml run --rm hurl_tests hurl --test *.hurl

# Run specific test file
docker compose --profile test -f docker-compose.dev.yml run --rm hurl_tests hurl --test auth.hurl

# Stop the application
docker compose -f docker-compose.dev.yml down
```

### Local Hurl Installation

If you have Hurl installed locally, you can run tests directly:

```bash
# Start the application first
docker compose -f docker-compose.dev.yml up -d

# Run tests with local Hurl
cd tests
hurl --test *.hurl

# Or run specific tests
hurl --test auth.hurl
hurl --test files.hurl --verbose  # with verbose output
```

## Test Structure

Each test file follows this pattern:

1. **Authentication** - Most endpoints require authentication, so tests start by logging in
2. **Capture Token** - The auth token is captured and used in subsequent requests
3. **API Tests** - Various API endpoints are tested with different scenarios
4. **Assertions** - Response status codes, headers, and body content are validated

## Authentication

The tests use the default admin credentials:
- Username: `admin`
- Password: `secret123` (from `ADMIN_PASSWORD` in docker-compose.dev.yml)

## Customizing Tests

### Adding New Tests

1. Create a new `.hurl` file in the `tests/` directory
2. Follow the authentication pattern from existing files
3. Add your API endpoint tests
4. Update the test script if needed

### Modifying Existing Tests

You can edit the `.hurl` files to:
- Add new assertions
- Test different request payloads
- Add performance testing with timing assertions
- Test error scenarios

### Environment Variables

You can use Hurl's variable features:

```bash
# Run with custom host
hurl --variable host=localhost:8000 --test *.hurl

# Run with custom credentials
hurl --variable username=admin --variable password=mypassword --test *.hurl
```

## Troubleshooting

### Application Not Ready

If tests fail because the application isn't ready:
- Use the test script which includes health checks
- Wait longer for the application to start
- Check logs: `./test.sh logs`

### Authentication Failures

If auth tests fail:
- Verify the `ADMIN_PASSWORD` in docker-compose.dev.yml
- Check if the password matches the one used in test files

### Network Issues

If connection fails:
- Ensure the application is running: `docker compose -f docker-compose.dev.yml ps`
- Check port mapping: port 8000 should be accessible
- Verify Docker networks are properly configured

### Test File Syntax

If Hurl complains about syntax:
- Check the Hurl documentation: https://hurl.dev/docs/
- Validate JSON payloads
- Ensure proper indentation and formatting

## Contributing

When adding new API endpoints:
1. Add corresponding Hurl tests
2. Update this README if needed
3. Test both success and failure scenarios
4. Include proper assertions for response validation