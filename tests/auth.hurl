# Authentication Tests
# Test login endpoint

POST http://localhost:8000/api/auth/
Content-Type: application/json
{
  "username": "admin",
  "password": "secret123"
}

HTTP 200
[Captures]
token: jsonpath "$.token"

# Test with invalid credentials
POST http://localhost:8000/api/auth/
Content-Type: application/json
{
  "username": "admin",
  "password": "wrongpassword"
}

HTTP 401