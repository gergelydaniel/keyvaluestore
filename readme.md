## Key-Value Store

This is a very simple and lightweight HTTP service, for storing key-value pairs in memory.

### Endpoints

 - GET `/:key`  
Returns the value with the given key from the map or `204` if the given key doesn't exist
 - POST `/:key`  
Sets the value with the given key to whatever is in the request's body

### Configuration

A config file named `keyvaluestore.ini` has to be in the working directory. Example:

```ini
[keyvaluestore]
port = 3000
read_token = basic_read_token
write_token = basic_write_token
```

The following configuration entries are required:
 - `port`: The port for the HTTP server to listen on
 - `read_token`: The `Bearer` token that will be expected for read operations
 - `write_token`: The `Bearer` token that will be expected for write operations
