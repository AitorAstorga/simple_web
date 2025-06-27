# Testing
Set an environment variable before execution with the `ADMIN_TOKEN`:
```bash
export ADMIN_TOKEN=secret123
```

# Syntax-by-Method
| Endpoint          | HTTP verb  | Mandatory inputs | Optional inputs | Body / form                                                                         |
| ----------------- | ---------- | ---------------- | --------------- | ----------------------------------------------------------------------------------- |
| **`/api/files`**  | **GET**    | –                | `path=<PATH>`   | –                                                                                   |
| **`/api/file`**   | **GET**    | `path=<PATH>`    | –               | –                                                                                   |
| **`/api/file`**   | **POST**   | `path=<PATH>`    | –               | JSON `{"content": <CONTENT>}`                                                       |
| **`/api/file`**   | **DELETE** | `path=<PATH>`    | –               | –                                                                                   |
| **`/api/move`**   | **POST**   | –                | –               | JSON `{"from": <SRC>, "to": <DST>}`                                                 |
| **`/api/upload`** | **POST**   | –                | –               | **multipart/form-data**<br>`files=@<LOCAL>;filename=<PATH>` (repeat for every file) |


- ```<AUTH>``` – admin token for the Authorization header (e.g. ```secret123```)
- ```<PATH>``` – relative path inside the public site (URL-encoded when in query strings)
- ```<CONTENT>``` – file contents as UTF-8 text
- ```<SRC>``` / ```<DST>``` – source and destination paths (same rules as ```<PATH>```)
- ```<LOCAL>``` – local filename on your machine to be uploaded

# API Examples
## List root
```bash
curl -H "Authorization: secret123" http://localhost:8000/api/files
```

## List a sub‑directory
```bash
curl -H "Authorization: secret123" "http://localhost:8000/api/files?path=img%2Ficons"
```

## Download a file
```bash
curl -H "Authorization: secret123" "http://localhost:8000/api/file?path=index.html"
```

## Save / create a file
```bash
curl -X POST \
     -H "Authorization: secret123" \
     -H "Content-Type: application/json" \
     -d '{"content":"console.log(\"hi\")"}' \
     "http://localhost:8000/api/file?path=js/app.js"
```

## Delete a file
```bash
curl -X DELETE -H "Authorization: secret123" \
     "http://localhost:8000/api/file?path=js/app.js"
```

## Move / rename a file
```bash
curl -X POST -H "Authorization: secret123" -H "Content-Type: application/json" \
     -d '{"from":"img/old.png","to":"img/new.png"}' \
     http://localhost:8000/api/move
```

## Upload multiple files / folders
```bash
curl -X POST -H "Authorization: secret123" \
     -F 'files=@docs/readme.md;filename=docs/readme.md' \
     -F 'files=@images/logo.svg;filename=images/logo.svg' \
     http://localhost:8000/api/upload
```