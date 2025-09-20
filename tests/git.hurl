# Git Operations API Tests
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

# Test git status endpoint
GET http://localhost:8000/api/git/status
Authorization: Bearer {{auth_token}}

HTTP 200

# Test auto-pull configuration get
GET http://localhost:8000/api/git/auto-pull
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
jsonpath "$.enabled" exists

# Test git test endpoint
GET http://localhost:8000/api/git/test
Authorization: Bearer {{auth_token}}

HTTP 200

# Test setting auto-pull configuration
POST http://localhost:8000/api/git/auto-pull
Authorization: Bearer {{auth_token}}
Content-Type: application/json
{
  "enabled": true,
  "interval_minutes": 60
}

HTTP 200