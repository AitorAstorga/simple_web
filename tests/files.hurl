# File Management API Tests
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

# Test listing files (root directory)
GET http://localhost:8000/api/files
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection
jsonpath "$[*].path" exists
jsonpath "$[*].is_dir" exists

# Test listing files with path parameter
GET http://localhost:8000/api/files?path=
Authorization: Bearer {{auth_token}}

HTTP 200

# Test getting a file (this might fail if file doesn't exist, but tests the endpoint)
GET http://localhost:8000/api/file?path=index.html
Authorization: Bearer {{auth_token}}

# Test saving a file
POST http://localhost:8000/api/file?path=test.txt
Authorization: Bearer {{auth_token}}
Content-Type: application/json
{
  "content": "Hello World from Hurl test!"
}

HTTP 200

# Test getting the file we just created
GET http://localhost:8000/api/file?path=test.txt
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
body contains "Hello World from Hurl test!"

# Test deleting the test file
DELETE http://localhost:8000/api/file?path=test.txt
Authorization: Bearer {{auth_token}}

HTTP 200