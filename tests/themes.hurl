# Theme Management API Tests
# First authenticate to get a token

POST http://localhost:8000/api/auth/
Content-Type: application/json
{
  "username": "admin",
  "password": "secret123"
}

HTTP 200
[Captures]
auth_token: jsonpath "$.token"

# Test listing themes
GET http://localhost:8000/api/themes
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection

# Test getting a specific theme (may fail if theme doesn't exist)
GET http://localhost:8000/api/theme?name=default
Authorization: Bearer {{auth_token}}

# Test saving a theme
POST http://localhost:8000/api/theme?name=test-theme
Authorization: Bearer {{auth_token}}
Content-Type: application/json
{
  "content": "/* Test theme CSS */\nbody { background: #f0f0f0; }"
}

HTTP 200

# Test getting the theme we just created
GET http://localhost:8000/api/theme?name=test-theme
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
body contains "Test theme CSS"

# Test deleting the test theme
DELETE http://localhost:8000/api/theme?name=test-theme
Authorization: Bearer {{auth_token}}

HTTP 200