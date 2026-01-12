# Theme Management API Tests
# First authenticate to get a token

POST http://simple_web:8000/api/auth/
Content-Type: application/json
{
  "username": "admin",
  "password": "secret123"
}

HTTP 200
[Captures]
auth_token: jsonpath "$.token"

# Test listing themes
GET http://simple_web:8000/api/themes
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection

# Test getting a specific theme (may fail if theme doesn't exist)
GET http://simple_web:8000/api/themes/default
Authorization: Bearer {{auth_token}}

# Test saving a theme
POST http://simple_web:8000/api/themes
Authorization: Bearer {{auth_token}}
Content-Type: application/json
{
  "name": "test-theme",
  "colors": {
    "background": "#f0f0f0",
    "foreground": "#333333",
    "accent": "#007bff"
  }
}

HTTP 200

# Test getting the theme we just created
GET http://simple_web:8000/api/themes/test-theme
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
body contains "test-theme"

# Test deleting the test theme
DELETE http://simple_web:8000/api/themes/test-theme
Authorization: Bearer {{auth_token}}

HTTP 200