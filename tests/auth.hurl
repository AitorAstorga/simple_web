# Authentication Tests
# Test login endpoint

POST http://simple_web:8000/api/auth/
Content-Type: application/json
{
  "username": "admin",
  "password": "secret123"
}

HTTP 200
[Captures]
token: jsonpath "$.token"

# Test with invalid credentials
POST http://simple_web:8000/api/auth/
Content-Type: application/json
{
  "username": "admin",
  "password": "wrongpassword"
}

HTTP 401